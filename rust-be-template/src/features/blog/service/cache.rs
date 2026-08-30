use tracing::{error, info};
use uuid::Uuid;

use super::{
    blog_service::BlogService,
    cache_policy::BLOG_POST_CACHE_MAX_ENTRIES,
};
use super::super::domain::cache::CachedPostInfo;

impl BlogService {
    fn normalize_slug(slug: &str) -> Option<String> {
        let slug = slug.trim().to_lowercase();
        (!slug.is_empty()).then_some(slug)
    }

    async fn remove_cached(&self, post_id: Uuid) -> Option<CachedPostInfo> {
        let removed = self
            .posts_cache
            .remove_async(&post_id)
            .await
            .map(|(_, post)| post);
        if let Some(post) = &removed
            && let Some(slug) = Self::normalize_slug(&post.post_slug)
            && let Some(mapped_id) = self.slug_cache.read_async(&slug, |_, id| *id).await
            && mapped_id == post_id
        {
            let _ = self.slug_cache.remove_async(&slug).await;
        }
        removed
    }

    async fn oldest_cached(&self) -> Option<(chrono::DateTime<chrono::Utc>, Uuid)> {
        let mut oldest = None;
        self.posts_cache
            .iter_async(|post_id, post| {
                let candidate = (post.post_created_at, *post_id);
                if oldest.is_none_or(|current| candidate < current) {
                    oldest = Some(candidate);
                }
                true
            })
            .await;
        oldest
    }

    async fn upsert_internal(&self, post: &CachedPostInfo, sync_search: bool) -> bool {
        let search_mutation = if sync_search {
            Some(self.search_index.lock_mutation().await)
        } else {
            None
        };
        let mutation = self.cache_mutation.lock().await;
        let previous = self
            .posts_cache
            .read_async(&post.post_id, |_, cached| cached.clone())
            .await;
        let mut admitted_new = false;
        if previous.is_none() {
            if self.posts_cache.len() >= BLOG_POST_CACHE_MAX_ENTRIES {
                let candidate = (post.post_created_at, post.post_id);
                match self.oldest_cached().await {
                    Some(oldest) if candidate <= oldest => {
                        self.metrics.record_rejected_admission();
                    }
                    Some((_, oldest_id)) => {
                        let _ = self.remove_cached(oldest_id).await;
                        self.metrics.record_eviction();
                        admitted_new = true;
                    }
                    None => admitted_new = true,
                }
            } else {
                admitted_new = true;
            }
        }
        if previous.is_some() || admitted_new {
            if let Some(previous) = previous
                && Self::normalize_slug(&previous.post_slug)
                    != Self::normalize_slug(&post.post_slug)
            {
                let _ = self.remove_cached(post.post_id).await;
            }
            let _ = self.posts_cache.upsert_async(post.post_id, post.clone()).await;
            if let Some(slug) = Self::normalize_slug(&post.post_slug) {
                if self.slug_cache.contains_async(&slug).await
                    || self.slug_cache.len() < BLOG_POST_CACHE_MAX_ENTRIES
                {
                    let _ = self.slug_cache.upsert_async(slug, post.post_id).await;
                } else {
                    self.metrics.record_rejected_admission();
                }
            }
        }
        drop(mutation);
        if sync_search {
            self.sync_search_document_locked(post).await;
        }
        drop(search_mutation);
        admitted_new
    }

    async fn sync_search_document_locked(&self, post: &CachedPostInfo) {
        let index = self.search_index.clone();
        let post_id = post.post_id;
        let title = post.post_title.clone();
        let tags = post.post_tags.clone();
        let published = post.post_is_published;
        let result = super::search::tasks::run_search_task(move || {
            if published {
                index.update_post(post_id, &title, &tags)
            } else {
                index.remove_post_and_commit(post_id)
            }
        })
        .await;
        if let Err(error_value) = result {
            error!(error = %error_value, post_id = %post.post_id, "Failed to update post search index");
        }
    }

    async fn rebuild_order(&self) {
        let mut ordered = Vec::with_capacity(self.posts_cache.len());
        self.posts_cache
            .iter_async(|post_id, post| {
                ordered.push((post.post_created_at, *post_id));
                true
            })
            .await;
        ordered.sort_by_key(|entry| std::cmp::Reverse(*entry));
        *self.order_cache.write().await = ordered.into_iter().map(|(_, id)| id).collect();
    }

    pub async fn synchronize_cache(&self) {
        let posts = match self
            .repository
            .recent_posts(BLOG_POST_CACHE_MAX_ENTRIES as i64)
            .await
        {
            Ok(posts) => posts,
            Err(error_value) => {
                error!(error = %error_value, "Could not synchronize post metadata cache");
                return;
            }
        };
        let mutation = self.cache_mutation.lock().await;
        self.posts_cache.clear_async().await;
        self.slug_cache.clear_async().await;
        drop(mutation);
        for post in &posts {
            let _ = self.upsert_internal(post, false).await;
        }
        self.rebuild_order().await;
        if let Err(error_value) = self.rebuild_search_index().await {
            error!(error = %error_value, "Failed to rebuild complete post search index");
        }
        let metrics = self.metrics.snapshot();
        info!(
            rows_synchronized = self.posts_cache.len(),
            slug_rows_synchronized = self.slug_cache.len(),
            max_entries = BLOG_POST_CACHE_MAX_ENTRIES,
            cache_hits = metrics.hits,
            cache_misses = metrics.misses,
            cache_evictions = metrics.evictions,
            rejected_admissions = metrics.rejected_admissions,
            database_read_throughs = metrics.database_read_throughs,
            "Post metadata cache synchronized"
        );
    }

    pub async fn insert_cache(&self, post: &CachedPostInfo) {
        if self.upsert_internal(post, true).await {
            self.rebuild_order().await;
        }
    }

    pub async fn insert_cache_without_search(&self, post: &CachedPostInfo) {
        if self.upsert_internal(post, false).await {
            self.rebuild_order().await;
        }
    }

    pub async fn delete_cache(&self, post_id: Uuid) {
        let search_mutation = self.search_index.lock_mutation().await;
        let mutation = self.cache_mutation.lock().await;
        let _ = self.remove_cached(post_id).await;
        drop(mutation);
        self.rebuild_order().await;
        let index = self.search_index.clone();
        if let Err(error_value) = super::search::tasks::run_search_task(move || {
            index.remove_post_and_commit(post_id)
        })
        .await
        {
            error!(error = %error_value, post_id = %post_id, "Failed to remove post from search index");
        }
        drop(search_mutation);
    }

    pub async fn update_cached_votes(&self, post_id: Uuid, upvotes: i64, downvotes: i64) {
        let _ = self.posts_cache.update_async(&post_id, |_, post| {
            post.total_upvotes = upvotes;
            post.total_downvotes = downvotes;
        }).await;
    }

    pub async fn update_cached_views(&self, post_id: Uuid, view_count: i64) {
        let _ = self.posts_cache.update_async(&post_id, |_, post| {
            post.post_view_count = view_count;
        }).await;
    }

    pub async fn cached_post(&self, post_id: &Uuid) -> Option<CachedPostInfo> {
        let post = self.posts_cache.read_async(post_id, |_, post| post.clone()).await;
        if post.is_some() { self.metrics.record_hit(); } else { self.metrics.record_miss(); }
        post
    }

    pub async fn cached_post_id_by_slug(&self, slug: &str) -> Option<Uuid> {
        let slug = Self::normalize_slug(slug)?;
        let post_id = self.slug_cache.read_async(&slug, |_, id| *id).await;
        if post_id.is_some() { self.metrics.record_hit(); } else { self.metrics.record_miss(); }
        post_id
    }

    pub async fn cache_slug(&self, slug: &str, post_id: Uuid) {
        let Some(slug) = Self::normalize_slug(slug) else { return };
        let mutation = self.cache_mutation.lock().await;
        if !self.posts_cache.contains_async(&post_id).await {
            self.metrics.record_rejected_admission();
        } else if self.slug_cache.contains_async(&slug).await
            || self.slug_cache.len() < BLOG_POST_CACHE_MAX_ENTRIES
        {
            let _ = self.slug_cache.upsert_async(slug, post_id).await;
        } else {
            self.metrics.record_rejected_admission();
        }
        drop(mutation);
    }
}
