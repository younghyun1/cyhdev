//! Keyset photograph comment pages with authors and viewer votes.

use std::collections::HashMap;

use diesel::{
    ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, QueryResult, SelectableHelper,
    sql_types::{Record, Timestamptz, Uuid as SqlUuid},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use jiff_diesel::ToDiesel;
use uuid::Uuid;

use crate::{
    features::{
        blog::domain::{
            comment_page::{CommentCursor, CommentPageRequest},
            vote::VoteState,
        },
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

/// One page of comment rows and the cursor of the page after it.
pub(super) type CommentRows = (Vec<PhotographCommentRecord>, Option<CommentCursor>);

impl PhotographyRepository {
    /// Reads a later comment page of an existing photograph.
    pub async fn comment_page(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<PhotographCommentPageData, PhotographyError> {
        let connection = self.connection().await?;
        let connection: &AsyncPgConnection = &connection;
        // The existence probe and the page do not depend on each other, so
        // both are sent before either result is read. A missing photograph
        // still runs the page query; its empty result is dropped.
        let photograph = async {
            photographs::table
                .filter(photographs::photograph_id.eq(photograph_id))
                .select(photographs::photograph_id)
                .first::<Uuid>(&mut &*connection)
                .await
                .optional()
        };
        let (photograph, rows) = tokio::try_join!(
            photograph,
            load_comment_rows(connection, photograph_id, request)
        )?;
        if photograph.is_none() {
            return Err(PhotographyError::PhotographNotFound);
        }
        finish_comment_page(connection, rows, viewer, &[]).await
    }
}

/// Loads one oldest-first page of comment rows. It depends only on the
/// photograph identifier, so a caller can join it with other first-wave reads.
pub(super) async fn load_comment_rows(
    connection: &AsyncPgConnection,
    photograph_id: Uuid,
    request: CommentPageRequest,
) -> QueryResult<CommentRows> {
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
                .gt((after.created_at.to_diesel(), after.comment_id)),
        );
    }
    let mut rows = query
        .limit(request.fetch_rows())
        .load::<PhotographCommentRecord>(&mut &*connection)
        .await?;
    let next_cursor = request.split_page(&mut rows, PhotographCommentRecord::cursor);
    Ok((rows, next_cursor))
}

/// Adds the batched author projection and the viewer's votes to loaded rows.
/// Both lookups depend on the rows but not on each other, so they share one
/// round trip. `extra_author_ids` joins the author batch, such as the
/// photograph owner.
pub(super) async fn finish_comment_page(
    connection: &AsyncPgConnection,
    (rows, next_cursor): CommentRows,
    viewer: Option<Uuid>,
    extra_author_ids: &[Uuid],
) -> Result<PhotographCommentPageData, PhotographyError> {
    let mut author_ids = rows
        .iter()
        .map(PhotographCommentRecord::author_id)
        .collect::<Vec<_>>();
    author_ids.extend_from_slice(extra_author_ids);
    let comments = rows
        .into_iter()
        .map(PhotographComment::from)
        .collect::<Vec<_>>();
    let comment_ids = comments
        .iter()
        .map(|comment| comment.photograph_comment_id)
        .collect::<Vec<_>>();
    let viewer_votes = async {
        match viewer {
            Some(user_id) if !comment_ids.is_empty() => {
                photograph_comment_votes::table
                    .filter(photograph_comment_votes::photograph_comment_id.eq_any(comment_ids))
                    .filter(photograph_comment_votes::user_id.eq(user_id))
                    .select((
                        photograph_comment_votes::photograph_comment_id,
                        photograph_comment_votes::is_upvote,
                    ))
                    .load::<(Uuid, bool)>(&mut &*connection)
                    .await
            }
            Some(_) | None => Ok(Vec::new()),
        }
    };
    let (authors, viewer_votes) =
        tokio::try_join!(load_public_authors(connection, &author_ids), viewer_votes)?;
    let votes = viewer_votes.into_iter().collect::<HashMap<_, _>>();
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
