use std::collections::HashMap;

use tracing::error;
use uuid::Uuid;

use super::blog_service::BlogService;
use super::super::{
    domain::{
        cache::CachedPostInfo,
        post::{PostInfoWithVote, UserBadgeInfo},
        vote::VoteState,
    },
    error::BlogError,
};

pub const BLOG_SEARCH_MAX_LIMIT: usize = 100;
pub const BLOG_SEARCH_MAX_OFFSET: usize = 10_000;
pub const BLOG_SEARCH_MAX_QUERY_CHARS: usize = 256;
pub const BLOG_SEARCH_MAX_TAGS: usize = 16;
pub const BLOG_SEARCH_MAX_TAG_CHARS: usize = 64;
const BLOG_LIST_MAX_PAGE: usize = 10_000;

impl BlogService {
    pub async fn recent_posts_for_compatibility(
        &self,
    ) -> Result<Vec<CachedPostInfo>, BlogError> {
        self.repository
            .recent_posts(super::cache_policy::BLOG_POST_CACHE_MAX_ENTRIES as i64)
            .await
    }

    pub async fn list_posts(
        &self,
        page: usize,
        page_size: usize,
        viewer_id: Option<Uuid>,
    ) -> (Vec<CachedPostInfo>, usize) {
        let page = page.clamp(1, BLOG_LIST_MAX_PAGE);
        let page_size = page_size.clamp(1, 100);
        self.metrics.record_database_read_through();
        match self
            .repository
            .list_posts(page, page_size, viewer_id)
            .await
        {
            Ok(page) => (page.posts, page.available_pages),
            Err(error_value) => {
                error!(error = %error_value, "PostgreSQL listing failed; using bounded metadata cache");
                // Authority lookup failed with the database query, so the cache
                // fallback must fail closed to published posts.
                self.bounded_cache_page(page, page_size, false)
                    .await
            }
        }
    }

    pub async fn bounded_cache_page(
        &self,
        page: usize,
        page_size: usize,
        include_unpublished: bool,
    ) -> (Vec<CachedPostInfo>, usize) {
        let page = page.clamp(1, BLOG_LIST_MAX_PAGE);
        let page_size = page_size.clamp(1, 100);
        let start = page.saturating_sub(1).saturating_mul(page_size);
        let ordered_ids = self.order_cache.read().await.clone();
        let mut posts = Vec::with_capacity(page_size);
        let mut visible = 0usize;
        for post_id in ordered_ids {
            let Some(post) = self.cached_post(&post_id).await else { continue };
            if !include_unpublished && !post.post_is_published {
                continue;
            }
            if visible >= start && posts.len() < page_size {
                posts.push(post);
            }
            visible += 1;
        }
        (posts, visible.div_ceil(page_size))
    }

    pub async fn present_posts(
        &self,
        posts: Vec<CachedPostInfo>,
        viewer_id: Option<Uuid>,
    ) -> Result<Vec<PostInfoWithVote>, BlogError> {
        if posts.is_empty() {
            return Ok(Vec::new());
        }
        let presentation = self.repository.presentation_data(&posts, viewer_id).await?;
        let country_flags = self.country_flags_for_authors(&presentation.authors).await;
        let result = posts
            .into_iter()
            .map(|post| {
                let vote_state = presentation
                    .votes
                    .get(&post.post_id)
                    .copied()
                    .unwrap_or(VoteState::DidNotVote);
                let (public_user_id, badge) = match presentation.authors.get(&post.user_id) {
                    Some(author) => {
                        let flag = author
                            .country_code()
                            .and_then(|code| country_flags.get(&code).cloned());
                        (
                            author.public_user_id(),
                            UserBadgeInfo::from_public_author(author, flag),
                        )
                    }
                    None => (Uuid::nil(), UserBadgeInfo::deleted()),
                };
                PostInfoWithVote::from_cached_info_with_vote(
                    post,
                    vote_state,
                    public_user_id,
                    badge,
                )
            })
            .collect();
        Ok(result)
    }

    async fn posts_from_ids(&self, post_ids: Vec<Uuid>) -> Result<Vec<CachedPostInfo>, BlogError> {
        let guards = self.lock_post_set(&post_ids).await;
        let mut by_id = HashMap::with_capacity(post_ids.len());
        let mut missing = Vec::new();
        for post_id in &post_ids {
            match self.cached_post(post_id).await {
                Some(post) => { by_id.insert(*post_id, post); }
                None => missing.push(*post_id),
            }
        }
        if !missing.is_empty() {
            self.metrics.record_database_read_through();
            let posts = self.repository.posts_by_ids(&missing).await?;
            for post in posts {
                by_id.insert(post.post_id, post);
            }
        }
        let posts = post_ids
            .iter()
            .filter_map(|post_id| by_id.get(post_id).cloned())
            .collect();
        drop(guards);
        Ok(posts)
    }

    async fn hydrate_search(
        &self,
        result: Result<(Vec<Uuid>, usize), BlogError>,
        operation: &'static str,
    ) -> Result<(Vec<CachedPostInfo>, usize), BlogError> {
        match result {
            Ok((ids, total)) => Ok((self.posts_from_ids(ids).await?, total)),
            Err(error_value) => {
                error!(error = %error_value, operation, "Post search failed");
                Err(error_value)
            }
        }
    }

    pub async fn search_title(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<CachedPostInfo>, usize), BlogError> {
        let (offset, limit) = validate_search_input(Some(query), &[], offset, limit)?;
        let search_query = self.search_index.lock_query().await;
        let index = self.search_index.clone();
        let query = query.to_owned();
        let result = super::search::tasks::run_search_task(move || {
            index.search_by_title_paged(&query, offset, limit)
        }).await;
        drop(search_query);
        self.hydrate_search(result, "title").await
    }

    pub async fn search_title_tags(
        &self,
        query: &str,
        tags: &[String],
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<CachedPostInfo>, usize), BlogError> {
        let (offset, limit) = validate_search_input(Some(query), tags, offset, limit)?;
        let search_query = self.search_index.lock_query().await;
        let index = self.search_index.clone();
        let query = query.to_owned();
        let tags = tags.to_vec();
        let result = super::search::tasks::run_search_task(move || {
            index.search_by_title_and_tags_paged(&query, &tags, offset, limit)
        }).await;
        drop(search_query);
        self.hydrate_search(result, "title_and_tags").await
    }

    pub async fn search_tags(
        &self,
        tags: &[String],
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<CachedPostInfo>, usize), BlogError> {
        let (offset, limit) = validate_search_input(None, tags, offset, limit)?;
        let search_query = self.search_index.lock_query().await;
        let index = self.search_index.clone();
        let tags = tags.to_vec();
        let result = super::search::tasks::run_search_task(move || {
            index.search_by_tags_paged(&tags, offset, limit)
        }).await;
        drop(search_query);
        self.hydrate_search(result, "tags").await
    }

    pub async fn search_tag(
        &self,
        tag: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<CachedPostInfo>, usize), BlogError> {
        let (offset, limit) = validate_search_input(Some(tag), &[], offset, limit)?;
        let search_query = self.search_index.lock_query().await;
        let index = self.search_index.clone();
        let tag = tag.to_owned();
        let result = super::search::tasks::run_search_task(move || {
            index.search_by_tag_paged(&tag, offset, limit)
        }).await;
        drop(search_query);
        self.hydrate_search(result, "tag").await
    }
}

fn validate_search_input(
    query: Option<&str>,
    tags: &[String],
    offset: usize,
    limit: usize,
) -> Result<(usize, usize), BlogError> {
    if query.is_some_and(|query| query.chars().count() > BLOG_SEARCH_MAX_QUERY_CHARS)
        || tags.len() > BLOG_SEARCH_MAX_TAGS
        || tags.iter().any(|tag| tag.chars().count() > BLOG_SEARCH_MAX_TAG_CHARS)
    {
        return Err(BlogError::InvalidInput);
    }
    Ok((offset.min(BLOG_SEARCH_MAX_OFFSET), limit.clamp(1, BLOG_SEARCH_MAX_LIMIT)))
}

#[cfg(test)]
mod tests {
    use super::{BLOG_SEARCH_MAX_LIMIT, BLOG_SEARCH_MAX_OFFSET, validate_search_input};

    #[test]
    fn search_window_is_bounded_before_index_work() {
        let window = validate_search_input(Some("rust"), &[], usize::MAX, usize::MAX);
        assert!(matches!(window, Ok((BLOG_SEARCH_MAX_OFFSET, BLOG_SEARCH_MAX_LIMIT))));
    }
}
