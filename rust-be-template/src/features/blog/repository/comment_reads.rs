//! Keyset comment pages and per-viewer presentation data on one connection.

use std::collections::HashMap;

use diesel::{
    ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, SelectableHelper,
    sql_types::{Record, Timestamptz, Uuid as SqlUuid},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
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

impl BlogRepository {
    /// Reads a later comment page; drafts are visible only to blog managers.
    pub async fn comment_page(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<CommentPageData, BlogError> {
        let mut connection = self.connection().await?;
        let published = posts::table
            .find(post_id)
            .select(posts::post_is_published)
            .first::<bool>(&mut connection)
            .await
            .optional()?
            .ok_or(BlogError::PostNotFound)?;
        if !published && !has_current_blog_authority(&mut connection, viewer_id).await? {
            return Err(BlogError::PostNotFound);
        }
        load_comment_page(&mut connection, post_id, viewer_id, request, &[]).await
    }

    /// Reads the first comment page, the post author, and the viewer's post
    /// vote with a single pool checkout. Visibility was already enforced by
    /// the post read that supplied `owner_id`.
    pub async fn post_social(
        &self,
        post_id: Uuid,
        owner_id: Uuid,
        viewer_id: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<PostSocialData, BlogError> {
        let mut connection = self.connection().await?;
        let page =
            load_comment_page(&mut connection, post_id, viewer_id, request, &[owner_id]).await?;
        let post_vote = match viewer_id {
            Some(viewer_id) => post_votes::table
                .filter(post_votes::post_id.eq(post_id))
                .filter(post_votes::user_id.eq(viewer_id))
                .select(post_votes::is_upvote)
                .first::<bool>(&mut connection)
                .await
                .optional()?
                .map_or(VoteState::DidNotVote, vote_state),
            None => VoteState::DidNotVote,
        };
        Ok(PostSocialData { page, post_vote })
    }
}

async fn load_comment_page(
    connection: &mut AsyncPgConnection,
    post_id: Uuid,
    viewer_id: Option<Uuid>,
    request: CommentPageRequest,
    extra_author_ids: &[Uuid],
) -> Result<CommentPageData, BlogError> {
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
                .gt((after.created_at, after.comment_id)),
        );
    }
    let mut rows = query
        .limit(request.fetch_rows())
        .load::<CommentRecord>(&mut *connection)
        .await?;
    let next_cursor = request.split_page(&mut rows, CommentRecord::cursor);
    let mut author_ids = rows
        .iter()
        .map(CommentRecord::author_id)
        .collect::<Vec<_>>();
    author_ids.extend_from_slice(extra_author_ids);
    let authors = load_public_authors(&mut *connection, &author_ids).await?;
    let comments = rows.into_iter().map(Comment::from).collect::<Vec<_>>();
    let votes = match viewer_id {
        Some(viewer_id) if !comments.is_empty() => {
            let comment_ids = comments
                .iter()
                .map(|comment| comment.comment_id)
                .collect::<Vec<_>>();
            comment_votes::table
                .filter(comment_votes::comment_id.eq_any(comment_ids))
                .filter(comment_votes::user_id.eq(viewer_id))
                .select((comment_votes::comment_id, comment_votes::is_upvote))
                .load::<(Uuid, bool)>(&mut *connection)
                .await?
                .into_iter()
                .map(|(comment_id, upvote)| (comment_id, vote_state(upvote)))
                .collect()
        }
        Some(_) | None => HashMap::new(),
    };
    Ok(CommentPageData {
        comments,
        next_cursor,
        authors,
        votes,
    })
}
