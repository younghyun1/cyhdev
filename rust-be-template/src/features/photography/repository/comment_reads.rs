//! Keyset photograph comment pages with authors and viewer votes.

use std::collections::HashMap;

use diesel::{
    ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, SelectableHelper,
    sql_types::{Record, Timestamptz, Uuid as SqlUuid},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::{
        blog::domain::{comment_page::CommentPageRequest, vote::VoteState},
        photography::{
            domain::social::{PhotographComment, PhotographCommentPageData},
            error::PhotographyError,
            repository::{
                photography_repository::PhotographyRepository, records::PhotographCommentRecord,
            },
        },
    },
    persistence::public_authors::load_public_authors,
    schema::{photograph_comment_votes, photograph_comments, photographs},
};

impl PhotographyRepository {
    /// Reads a later comment page of an existing photograph.
    pub async fn comment_page(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<PhotographCommentPageData, PhotographyError> {
        let mut connection = self.connection().await?;
        photographs::table
            .filter(photographs::photograph_id.eq(photograph_id))
            .select(photographs::photograph_id)
            .first::<Uuid>(&mut connection)
            .await
            .optional()?
            .ok_or(PhotographyError::PhotographNotFound)?;
        load_comment_page(&mut connection, photograph_id, viewer, request, &[]).await
    }
}

/// Loads one oldest-first page on the caller's connection. `extra_author_ids`
/// joins the same batched author projection, such as the photograph owner.
pub(super) async fn load_comment_page(
    connection: &mut AsyncPgConnection,
    photograph_id: Uuid,
    viewer: Option<Uuid>,
    request: CommentPageRequest,
    extra_author_ids: &[Uuid],
) -> Result<PhotographCommentPageData, PhotographyError> {
    let mut query = photograph_comments::table
        .filter(photograph_comments::photograph_id.eq(photograph_id))
        .select(PhotographCommentRecord::as_select())
        .order((
            photograph_comments::photograph_comment_created_at.asc(),
            photograph_comments::photograph_comment_id.asc(),
        ))
        .into_boxed();
    if let Some(after) = request.after() {
        // A row comparison bounds photograph_comments_photograph_page_idx
        // (photograph_id, photograph_comment_created_at, photograph_comment_id).
        query = query.filter(
            (
                photograph_comments::photograph_comment_created_at,
                photograph_comments::photograph_comment_id,
            )
                .into_sql::<Record<(Timestamptz, SqlUuid)>>()
                .gt((after.created_at, after.comment_id)),
        );
    }
    let mut rows = query
        .limit(request.fetch_rows())
        .load::<PhotographCommentRecord>(&mut *connection)
        .await?;
    let next_cursor = request.split_page(&mut rows, PhotographCommentRecord::cursor);
    let mut author_ids = rows
        .iter()
        .map(PhotographCommentRecord::author_id)
        .collect::<Vec<_>>();
    author_ids.extend_from_slice(extra_author_ids);
    let authors = load_public_authors(&mut *connection, &author_ids).await?;
    let comments = rows
        .into_iter()
        .map(PhotographComment::from)
        .collect::<Vec<_>>();
    let votes = match viewer {
        Some(user_id) if !comments.is_empty() => {
            let comment_ids = comments
                .iter()
                .map(|comment| comment.photograph_comment_id)
                .collect::<Vec<_>>();
            photograph_comment_votes::table
                .filter(photograph_comment_votes::photograph_comment_id.eq_any(comment_ids))
                .filter(photograph_comment_votes::user_id.eq(user_id))
                .select((
                    photograph_comment_votes::photograph_comment_id,
                    photograph_comment_votes::is_upvote,
                ))
                .load::<(Uuid, bool)>(&mut *connection)
                .await?
                .into_iter()
                .collect::<HashMap<_, _>>()
        }
        Some(_) | None => HashMap::new(),
    };
    let comments = comments
        .into_iter()
        .map(|comment| {
            let state = match votes.get(&comment.photograph_comment_id) {
                Some(true) => VoteState::Upvoted,
                Some(false) => VoteState::Downvoted,
                None => VoteState::DidNotVote,
            };
            (comment, state)
        })
        .collect();
    Ok(PhotographCommentPageData {
        comments,
        next_cursor,
        authors,
    })
}
