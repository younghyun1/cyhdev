//! Post detail and comment page reads.

use std::collections::HashMap;

use uuid::Uuid;

use super::super::{
    domain::{
        cache::CachedPostInfo,
        comment::{CommentPage, CommentResponse},
        comment_page::CommentPageRequest,
        post::{PostInfo, PostLookup, ReadPostResult, UserBadgeInfo},
        vote::VoteState,
    },
    error::BlogError,
    repository::comment_reads::CommentPageData,
};
use super::blog_service::BlogService;

impl BlogService {
    /// Reads one post with its first comment page and records one view.
    ///
    /// Two pool checkouts serve the whole read: the post (with tags on a cache
    /// miss) under the post stripe, then comments, authors, and the viewer's
    /// votes. The view is buffered after the stripe is released.
    pub async fn read_post(
        &self,
        lookup: PostLookup,
        viewer_id: Option<Uuid>,
    ) -> Result<ReadPostResult, BlogError> {
        let post_id = self.resolve_post_lookup(&lookup).await?;
        let view_epoch = self.views.stable_epoch().await;
        let post_use_case = self.lock_post_use_case(post_id).await;
        let cached = self.cached_post(&post_id).await;
        let read = self
            .repository
            .read_post(post_id, viewer_id, cached.is_none())
            .await?;
        let post_tags = match (read.tags, cached) {
            (Some(tags), _) => tags,
            (None, Some(cached)) => cached.post_tags,
            (None, None) => Vec::new(),
        };
        let mut post = read.post;
        // `post_content` is rendered at write time and remains authoritative.
        // Reads refresh cached metadata only; the search index is rebuilt at
        // startup and updated by writes, so a read never commits the index.
        let cached = CachedPostInfo::from_post_info_with_tags(
            PostInfo::from(post.clone()),
            post_tags.clone(),
        );
        self.insert_cache_without_search(&cached).await;
        drop(post_use_case);

        let social = self
            .repository
            .post_social(
                post_id,
                post.user_id,
                viewer_id,
                CommentPageRequest::first_page(),
            )
            .await?;
        let country_flags = self.country_flags_for_authors(&social.page.authors).await;
        let (public_owner_id, post_badge) =
            UserBadgeInfo::resolve(&social.page.authors, &country_flags, post.user_id);
        post.user_id = public_owner_id;
        let comments_next_cursor = social.page.next_cursor;
        let comments = present_comments(social.page, &country_flags);
        post.post_view_count = self
            .record_and_count_view(post_id, post.post_view_count, view_epoch)
            .await;
        self.update_cached_views(post_id, post.post_view_count)
            .await;
        Ok(ReadPostResult {
            post,
            post_tags,
            comments,
            comments_next_cursor,
            vote_state: social.post_vote,
            user_badge_info: post_badge,
        })
    }

    /// Reads a later comment page of a post the viewer may see.
    pub async fn comment_page(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
        request: CommentPageRequest,
    ) -> Result<CommentPage, BlogError> {
        let page = self
            .repository
            .comment_page(post_id, viewer_id, request)
            .await?;
        let country_flags = self.country_flags_for_authors(&page.authors).await;
        let next_cursor = page.next_cursor;
        Ok(CommentPage {
            comments: present_comments(page, &country_flags),
            next_cursor,
        })
    }

    async fn resolve_post_lookup(&self, lookup: &PostLookup) -> Result<Uuid, BlogError> {
        match lookup {
            PostLookup::Id(post_id) => Ok(*post_id),
            PostLookup::Slug(slug) => match self.cached_post_id_by_slug(slug).await {
                Some(post_id) => Ok(post_id),
                None => {
                    let post_id = self
                        .repository
                        .resolve_post_id(lookup)
                        .await?
                        .ok_or(BlogError::PostNotFound)?;
                    self.cache_slug(slug, post_id).await;
                    Ok(post_id)
                }
            },
        }
    }
}

fn present_comments(
    page: CommentPageData,
    country_flags: &HashMap<i32, String>,
) -> Vec<CommentResponse> {
    page.comments
        .into_iter()
        .map(|comment| {
            let vote = page
                .votes
                .get(&comment.comment_id)
                .copied()
                .unwrap_or(VoteState::DidNotVote);
            let (public_id, badge) =
                UserBadgeInfo::resolve(&page.authors, country_flags, comment.user_id);
            CommentResponse::from_comment_votestate_and_badge_info(comment, vote, public_id, badge)
        })
        .collect()
}
