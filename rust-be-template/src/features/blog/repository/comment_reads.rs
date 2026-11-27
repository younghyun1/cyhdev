//! Keyset comment pages and per-viewer presentation data on one connection.

use std::collections::HashMap;

use diesel::{
    ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, QueryResult, SelectableHelper,
    sql_types::{Record, Timestamptz, Uuid as SqlUuid},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use jiff_diesel::ToDiesel;
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    persistence::public_authors::load_public_authors,
    schema::{comment_votes, comments, post_votes, posts},
};

use super::super::{
    domain::{
        comment::Comment,
        comment_page::{CommentCursor, CommentPageRequest},
        vote::VoteState,
    },
    error::BlogError,
};
use super::{
    authority::has_current_blog_authority, blog_repository::BlogRepository, queries::vote_state,
    records::CommentRecord,
};

/// One comment page with every author and the viewer's votes on it.
pub struct CommentPageData {
    pub comments: Vec<Comment>,
    pub next_cursor: Option<CommentCursor>,
    pub authors: HashMap<Uuid, PublicAuthor>,
    pub votes: HashMap<Uuid, VoteState>,
}

/// The social half of a post detail read.
pub struct PostSocialData {
    pub page: CommentPageData,
    pub post_vote: VoteState,
}

/// One page of comment rows and the cursor of the page after it.
type CommentRows = (Vec<CommentRecord>, Option<CommentCursor>);

impl BlogRepository {
    /// Reads a later comment page; drafts are visible only to blog managers.
    pub async fn comment_page(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<CommentPageData, BlogError> {
        let connection = self.connection().await?;
        let connection: &AsyncPgConnection = &connection;
        // The visibility probe and the page are sent together. The rows stay
        // in this function until the probe, and for a draft the authority
        // check, admit the viewer; a refused read drops them unseen.
        let published = async {
            posts::table
                .find(post_id)
                .select(posts::post_is_published)
                .first::<bool>(&mut &*connection)
                .await
                .optional()
        };
        let (published, rows) =
            tokio::try_join!(published, load_comment_rows(connection, post_id, request))?;
        let published = published.ok_or(BlogError::PostNotFound)?;
        if !published && !has_current_blog_authority(connection, viewer_id).await? {
            return Err(BlogError::PostNotFound);
        }
        finish_comment_page(connection, rows, viewer_id, &[]).await
    }

    /// Reads the first comment page, the post author, and the viewer's post
    /// vote with a single pool checkout and two round trips. Visibility was
    /// already enforced by the post read that supplied `owner_id`.
    pub async fn post_social(
        &self,
        post_id: Uuid,
        owner_id: Uuid,
        viewer_id: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<PostSocialData, BlogError> {
        let connection = self.connection().await?;
        let connection: &AsyncPgConnection = &connection;
        let page = async {
            let rows = load_comment_rows(connection, post_id, request).await?;
            finish_comment_page(connection, rows, viewer_id, &[owner_id]).await
        };
        let post_vote = async {
            match viewer_id {
                Some(viewer_id) => post_votes::table
                    .filter(post_votes::post_id.eq(post_id))
                    .filter(post_votes::user_id.eq(viewer_id))
                    .select(post_votes::is_upvote)
                    .first::<bool>(&mut &*connection)
                    .await
                    .optional()
                    .map_err(BlogError::Database),
                None => Ok(None),
            }
        };
        // The post vote rides with the comment rows; the author batch and the
        // comment votes follow once the rows are known.
        let (page, post_vote) = tokio::try_join!(page, post_vote)?;
        Ok(PostSocialData {
            page,
            post_vote: post_vote.map_or(VoteState::DidNotVote, vote_state),
        })
    }
}

/// Loads one oldest-first page of comment rows. It depends only on the post
/// identifier, so a caller can join it with other first-wave reads.
async fn load_comment_rows(
    connection: &AsyncPgConnection,
    post_id: Uuid,
    request: CommentPageRequest,
) -> QueryResult<CommentRows> {
    let mut query = comments::table
        .filter(comments::post_id.eq(post_id))
        .select(CommentRecord::as_select())
        .order((
            comments::comment_created_at.asc(),
            comments::comment_id.asc(),
        ))
        .into_boxed();
    if let Some(after) = request.after() {
        // A row comparison, unlike an OR chain, is a range bound on
        // comments_post_page_idx (post_id, comment_created_at, comment_id).
        query = query.filter(
            (comments::comment_created_at, comments::comment_id)
                .into_sql::<Record<(Timestamptz, SqlUuid)>>()
                .gt((after.created_at.to_diesel(), after.comment_id)),
        );
    }
    let mut rows = query
        .limit(request.fetch_rows())
        .load::<CommentRecord>(&mut &*connection)
        .await?;
    let next_cursor = request.split_page(&mut rows, CommentRecord::cursor);
    Ok((rows, next_cursor))
}

/// Adds the batched author projection and the viewer's votes to loaded rows.
/// Both lookups depend on the rows but not on each other, so they share one
/// round trip. `extra_author_ids` joins the author batch, such as the post
/// owner.
async fn finish_comment_page(
    connection: &AsyncPgConnection,
    (rows, next_cursor): CommentRows,
    viewer_id: Option<Uuid>,
    extra_author_ids: &[Uuid],
) -> Result<CommentPageData, BlogError> {
    let mut author_ids = rows
        .iter()
        .map(CommentRecord::author_id)
        .collect::<Vec<_>>();
    author_ids.extend_from_slice(extra_author_ids);
    let comments = rows.into_iter().map(Comment::from).collect::<Vec<_>>();
    let comment_ids = comments
        .iter()
        .map(|comment| comment.comment_id)
        .collect::<Vec<_>>();
    let viewer_votes = async {
        match viewer_id {
            Some(viewer_id) if !comment_ids.is_empty() => {
                comment_votes::table
                    .filter(comment_votes::comment_id.eq_any(comment_ids))
                    .filter(comment_votes::user_id.eq(viewer_id))
                    .select((comment_votes::comment_id, comment_votes::is_upvote))
                    .load::<(Uuid, bool)>(&mut &*connection)
                    .await
            }
            Some(_) | None => Ok(Vec::new()),
        }
    };
    let (authors, viewer_votes) =
        tokio::try_join!(load_public_authors(connection, &author_ids), viewer_votes)?;
    let votes = viewer_votes
        .into_iter()
        .map(|(comment_id, upvote)| (comment_id, vote_state(upvote)))
        .collect();
    Ok(CommentPageData {
        comments,
        next_cursor,
        authors,
        votes,
    })
}
