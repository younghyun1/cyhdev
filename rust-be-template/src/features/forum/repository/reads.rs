//! Keyset-paginated public topic and reply reads.

use diesel::{
    ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, SelectableHelper,
    dsl::exists,
    sql_types::{Bool, Integer, Record, Text, Timestamptz, Uuid as SqlUuid},
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::{
        accounts::domain::public_author::PublicAuthor,
        forum::{
            domain::{
                models::{
                    ForumReply, ForumReplyCursor, ForumReplyPage, ForumReplyView, ForumTopicCursor,
                    ForumTopicDetail, ForumTopicPage, ForumTopicView,
                },
                validation::{ForumPageSize, ForumSearch},
            },
            error::ForumError,
            repository::{
                forum_repository::ForumRepository,
                records::{ForumReplyRecord, ForumTopicHeadRecord, ForumTopicRecord},
                search::{ForumSearchMatches, forum_websearch_to_tsquery},
            },
        },
    },
    persistence::public_authors::load_public_authors,
    schema::{forum_replies, forum_topic_subscriptions, forum_topics},
};

/// Characters of each topic body a list card needs: the card shows 300 and
/// the extra character tells it whether to append an ellipsis.
const FORUM_TOPIC_SUMMARY_CHARS: i32 = 301;

diesel::define_sql_function! {
    /// PostgreSQL's built-in `left(text, integer)`, which counts characters.
    #[sql_name = "left"]
    fn text_left(text: Text, length: Integer) -> Text;
}

impl ForumRepository {
    /// Lists topics with bodies truncated to [`FORUM_TOPIC_SUMMARY_CHARS`]
    /// characters, so a page never transfers full 20,000-character bodies.
    pub async fn topic_page(
        &self,
        search: Option<&ForumSearch>,
        before: Option<ForumTopicCursor>,
        page_size: ForumPageSize,
    ) -> Result<ForumTopicPage, ForumError> {
        let mut connection = self.connection().await?;
        let mut query = forum_topics::table
            .select((
                ForumTopicHeadRecord::as_select(),
                text_left(forum_topics::forum_topic_body, FORUM_TOPIC_SUMMARY_CHARS),
            ))
            .into_boxed();
        if let Some(search) = search {
            query =
                query
                    .filter(forum_topics::forum_topic_content_state.eq(
                        crate::features::forum::repository::enums::DbForumContentState::Visible,
                    ))
                    .filter(ForumSearchMatches::new(
                        forum_topics::forum_topic_search_vector,
                        forum_websearch_to_tsquery(search.as_ref()),
                    ))
                    .order((
                        forum_topics::forum_topic_last_activity_at.desc(),
                        forum_topics::forum_topic_id.desc(),
                    ));
        } else {
            query = query.order((
                forum_topics::forum_topic_is_pinned.desc(),
                forum_topics::forum_topic_last_activity_at.desc(),
                forum_topics::forum_topic_id.desc(),
            ));
        }
        // Row comparisons, unlike OR chains, bound the scan of the matching
        // all-descending keyset index: forum_topics_visible_recent_idx for
        // search and forum_topics_public_page_idx for the pinned-first list.
        if let Some(cursor) = before {
            query = if search.is_some() {
                query.filter(
                    (
                        forum_topics::forum_topic_last_activity_at,
                        forum_topics::forum_topic_id,
                    )
                        .into_sql::<Record<(Timestamptz, SqlUuid)>>()
                        .lt((cursor.last_activity_at, cursor.topic_id)),
                )
            } else {
                query.filter(
                    (
                        forum_topics::forum_topic_is_pinned,
                        forum_topics::forum_topic_last_activity_at,
                        forum_topics::forum_topic_id,
                    )
                        .into_sql::<Record<(Bool, Timestamptz, SqlUuid)>>()
                        .lt((
                            cursor.is_pinned,
                            cursor.last_activity_at,
                            cursor.topic_id,
                        )),
                )
            };
        }
        let mut records = query
            .limit(i64::from(page_size.into_inner()) + 1)
            .load::<(ForumTopicHeadRecord, String)>(&mut connection)
            .await?
            .into_iter()
            .map(|(head, summary)| head.with_body(summary))
            .collect::<Vec<_>>();
        let next_cursor = topic_next_cursor(&mut records, page_size);
        let author_ids = records
            .iter()
            .map(|record| record.forum_topic_author_user_id)
            .collect::<Vec<_>>();
        let authors = load_public_authors(&mut connection, &author_ids).await?;
        let items = records
            .into_iter()
            .map(|record| {
                let author = authors
                    .get(&record.forum_topic_author_user_id)
                    .cloned()
                    .unwrap_or_else(PublicAuthor::deleted);
                ForumTopicView {
                    topic: record.into(),
                    author,
                }
            })
            .collect();
        Ok(ForumTopicPage { items, next_cursor })
    }

    pub async fn topic_detail(
        &self,
        topic_id: Uuid,
        viewer_user_id: Option<Uuid>,
        after: Option<ForumReplyCursor>,
        page_size: ForumPageSize,
    ) -> Result<ForumTopicDetail, ForumError> {
        let mut connection = self.connection().await?;
        let topic_record = forum_topics::table
            .filter(forum_topics::forum_topic_id.eq(topic_id))
            .select(ForumTopicRecord::as_select())
            .first::<ForumTopicRecord>(&mut connection)
            .await
            .optional()?
            .ok_or(ForumError::TopicNotFound)?;
        let topic_visible = topic_record.forum_topic_content_state
            == crate::features::forum::repository::enums::DbForumContentState::Visible;
        let mut reply_query = forum_replies::table
            .filter(forum_replies::forum_reply_topic_id.eq(topic_id))
            .select(ForumReplyRecord::as_select())
            .order((
                forum_replies::forum_reply_created_at.asc(),
                forum_replies::forum_reply_id.asc(),
            ))
            .into_boxed();
        if let Some(cursor) = after {
            // Bounds forum_replies_topic_page_idx (topic, created_at, reply).
            reply_query = reply_query.filter(
                (
                    forum_replies::forum_reply_created_at,
                    forum_replies::forum_reply_id,
                )
                    .into_sql::<Record<(Timestamptz, SqlUuid)>>()
                    .gt((cursor.created_at, cursor.reply_id)),
            );
        }
        let mut reply_records = reply_query
            .limit(i64::from(page_size.into_inner()) + 1)
            .load::<ForumReplyRecord>(&mut connection)
            .await?;
        let next_cursor = reply_next_cursor(&mut reply_records, page_size);
        let is_subscribed = match viewer_user_id {
            Some(user_id) => {
                diesel::select(exists(
                    forum_topic_subscriptions::table
                        .filter(
                            forum_topic_subscriptions::forum_topic_subscription_topic_id
                                .eq(topic_id),
                        )
                        .filter(
                            forum_topic_subscriptions::forum_topic_subscription_user_id.eq(user_id),
                        ),
                ))
                .get_result::<bool>(&mut connection)
                .await?
            }
            None => false,
        };
        let mut author_ids = Vec::with_capacity(reply_records.len() + 1);
        author_ids.push(topic_record.forum_topic_author_user_id);
        author_ids.extend(
            reply_records
                .iter()
                .map(|record| record.forum_reply_author_user_id),
        );
        let authors = load_public_authors(&mut connection, &author_ids).await?;
        let topic_author = authors
            .get(&topic_record.forum_topic_author_user_id)
            .cloned()
            .unwrap_or_else(PublicAuthor::deleted);
        let topic = ForumTopicView {
            topic: topic_record.into(),
            author: topic_author,
        };
        let replies = reply_records
            .into_iter()
            .map(|record| {
                let author = authors
                    .get(&record.forum_reply_author_user_id)
                    .cloned()
                    .unwrap_or_else(PublicAuthor::deleted);
                let mut reply: ForumReply = record.into();
                if !topic_visible {
                    reply.body = None;
                }
                ForumReplyView { reply, author }
            })
            .collect();
        Ok(ForumTopicDetail {
            topic,
            replies: ForumReplyPage {
                items: replies,
                next_cursor,
            },
            is_subscribed,
        })
    }
}

fn topic_next_cursor(
    records: &mut Vec<ForumTopicRecord>,
    page_size: ForumPageSize,
) -> Option<ForumTopicCursor> {
    if records.len() <= usize::from(page_size.into_inner()) {
        return None;
    }
    records.pop();
    records.last().map(|record| ForumTopicCursor {
        is_pinned: record.forum_topic_is_pinned,
        last_activity_at: record.forum_topic_last_activity_at,
        topic_id: record.forum_topic_id,
    })
}

fn reply_next_cursor(
    records: &mut Vec<ForumReplyRecord>,
    page_size: ForumPageSize,
) -> Option<ForumReplyCursor> {
    if records.len() <= usize::from(page_size.into_inner()) {
        return None;
    }
    records.pop();
    records.last().map(|record| ForumReplyCursor {
        created_at: record.forum_reply_created_at,
        reply_id: record.forum_reply_id,
    })
}
