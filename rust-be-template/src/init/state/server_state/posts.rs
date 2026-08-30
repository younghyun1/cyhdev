use tracing::{error, info};
use uuid::Uuid;

use super::{ServerState, blog_cache_policy::BLOG_POST_CACHE_MAX_ENTRIES};
use crate::domain::blog::blog::CachedPostInfo;
use crate::init::load_cache::post_info::load_post_info;
use crate::util::time::now::tokio_now;

impl ServerState {
    fn normalize_post_slug(slug: &str) -> Option<String> {
        let normalized = slug.trim().to_lowercase();
        if normalized.is_empty() { None } else { Some(normalized) }
    }

    async fn remove_cached_post(&self, post_id: Uuid) -> Option<CachedPostInfo> {
        let removed = self.blog_posts_cache.remove_async(&post_id).await.map(|(_, post)| post);
        if let Some(post) = &removed
            && let Some(slug) = Self::normalize_post_slug(&post.post_slug)
            && let Some(mapped_id) = self
                .blog_post_slug_cache
                .read_async(&slug, |_, cached_id| *cached_id)
                .await
            && mapped_id == post_id
        {
            let _ = self.blog_post_slug_cache.remove_async(&slug).await;
        }
        removed
    }

    async fn oldest_cached_post(&self) -> Option<(chrono::DateTime<chrono::Utc>, Uuid)> {
        let mut oldest = None;
        self.blog_posts_cache
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

    async fn upsert_post_cache_internal(
        &self,
        post: &CachedPostInfo,
        sync_search_index: bool,
    ) -> bool {
        let mutation = self.blog_cache_mutation.lock().await;
        let previous = self
            .blog_posts_cache
            .read_async(&post.post_id, |_, cached| cached.clone())
            .await;
        let mut admitted_new = false;

        if previous.is_none() {
            if self.blog_posts_cache.len() >= BLOG_POST_CACHE_MAX_ENTRIES {
                let candidate = (post.post_created_at, post.post_id);
                match self.oldest_cached_post().await {
                    Some(oldest) if candidate <= oldest => {
                        self.blog_cache_metrics.record_rejected_admission();
                    }
                    Some((_, oldest_id)) => {
                        let _ = self.remove_cached_post(oldest_id).await;
                        self.blog_cache_metrics.record_eviction();
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
                && Self::normalize_post_slug(&previous.post_slug)
                    != Self::normalize_post_slug(&post.post_slug)
            {
                let _ = self.remove_cached_post(post.post_id).await;
            }
            let _ = self.blog_posts_cache.upsert_async(post.post_id, post.clone()).await;
            if let Some(slug) = Self::normalize_post_slug(&post.post_slug) {
                if self.blog_post_slug_cache.contains_async(&slug).await
                    || self.blog_post_slug_cache.len() < BLOG_POST_CACHE_MAX_ENTRIES
                {
                    let _ = self.blog_post_slug_cache.upsert_async(slug, post.post_id).await;
                } else {
                    self.blog_cache_metrics.record_rejected_admission();
                }
            }
        }
        drop(mutation);

        if sync_search_index {
            self.sync_post_search_document(post);
        }
        admitted_new
    }

    fn sync_post_search_document(&self, post: &CachedPostInfo) {
        let result = if post.post_is_published {
            self.search_index.update_post(post.post_id, &post.post_title, &post.post_tags)
        } else {
            self.search_index.remove_post_and_commit(post.post_id)
        };
        if let Err(e) = result {
            error!(error = ?e, post_id = %post.post_id, "Failed to update search index");
        }
    }

    async fn rebuild_post_order_cache(&self) {
        let mut ordered = Vec::with_capacity(self.blog_posts_cache.len());
        self.blog_posts_cache
            .iter_async(|post_id, post| {
                ordered.push((post.post_created_at, *post_id));
                true
            })
            .await;
        ordered.sort_by_key(|entry| std::cmp::Reverse(*entry));
        *self.blog_post_order_cache.write().await =
            ordered.into_iter().map(|(_, post_id)| post_id).collect();
    }

    pub async fn synchronize_post_info_cache(&self) {
        let start = tokio_now();
        let post_info = match load_post_info(self).await {
            Ok(posts) => posts,
            Err(e) => {
                error!(error = ?e, "Could not synchronize post metadata cache");
                return;
            }
        };

        let mutation = self.blog_cache_mutation.lock().await;
        self.blog_posts_cache.clear_async().await;
        self.blog_post_slug_cache.clear_async().await;
        drop(mutation);
        for post in &post_info {
            let _ = self.upsert_post_cache_internal(post, false).await;
        }
        self.rebuild_post_order_cache().await;

        if let Err(e) = self.rebuild_post_search_index_from_db().await {
            error!(error = ?e, "Failed to rebuild complete post search index");
        }

        let metrics = self.blog_cache_metrics.snapshot();
        info!(
            rows_synchronized = self.blog_posts_cache.len(),
            slug_rows_synchronized = self.blog_post_slug_cache.len(),
            max_entries = BLOG_POST_CACHE_MAX_ENTRIES,
            cache_hits = metrics.hits,
            cache_misses = metrics.misses,
            cache_evictions = metrics.evictions,
            rejected_admissions = metrics.rejected_admissions,
            database_read_throughs = metrics.database_read_throughs,
            elapsed = ?start.elapsed(),
            "Post metadata cache synchronized"
        );
    }

    pub(super) async fn get_posts_from_bounded_cache(
        &self,
        page: usize,
        page_size: usize,
        include_unpublished: bool,
    ) -> (Vec<CachedPostInfo>, usize) {
        let page_size = page_size.clamp(1, 100);
        let start_index = page.saturating_sub(1).saturating_mul(page_size);
        let ordered_ids = self.blog_post_order_cache.read().await.clone();
        let mut posts = Vec::with_capacity(page_size);
        let mut visible = 0usize;
        for post_id in ordered_ids {
            let post = match self.get_post_from_cache(&post_id).await {
                Some(post) => post,
                None => continue,
            };
            if !include_unpublished && !post.post_is_published { continue; }
            if visible >= start_index && posts.len() < page_size { posts.push(post); }
            visible += 1;
        }
        (posts, visible.div_ceil(page_size))
    }

    pub async fn delete_post_from_cache(&self, post_id: Uuid) {
        let mutation = self.blog_cache_mutation.lock().await;
        let _ = self.remove_cached_post(post_id).await;
        drop(mutation);
        self.rebuild_post_order_cache().await;
        if let Err(e) = self.search_index.remove_post_and_commit(post_id) {
            error!(error = ?e, post_id = %post_id, "Failed to remove post from search index");
        }
    }

    pub async fn insert_post_to_cache(&self, post: &CachedPostInfo) {
        if self.upsert_post_cache_internal(post, true).await {
            self.rebuild_post_order_cache().await;
        }
    }

    pub async fn insert_post_to_cache_without_search_sync(&self, post: &CachedPostInfo) {
        if self.upsert_post_cache_internal(post, false).await {
            self.rebuild_post_order_cache().await;
        }
    }

    pub async fn update_post_vote_counts(&self, post_id: Uuid, upvotes: i64, downvotes: i64) {
        let _ = self.blog_posts_cache.update_async(&post_id, |_, cached| {
            cached.total_upvotes = upvotes;
            cached.total_downvotes = downvotes;
        }).await;
    }

    pub async fn get_post_from_cache(&self, post_id: &Uuid) -> Option<CachedPostInfo> {
        let post = self.blog_posts_cache.read_async(post_id, |_, post| post.clone()).await;
        if post.is_some() { self.blog_cache_metrics.record_hit(); }
        else { self.blog_cache_metrics.record_miss(); }
        post
    }

    pub async fn get_post_id_by_slug_from_cache(&self, slug: &str) -> Option<Uuid> {
        let normalized = Self::normalize_post_slug(slug)?;
        let post_id = self.blog_post_slug_cache.read_async(&normalized, |_, id| *id).await;
        if post_id.is_some() { self.blog_cache_metrics.record_hit(); }
        else { self.blog_cache_metrics.record_miss(); }
        post_id
    }

    pub async fn cache_post_slug_mapping(&self, slug: &str, post_id: Uuid) {
        let Some(normalized) = Self::normalize_post_slug(slug) else { return };
        let mutation = self.blog_cache_mutation.lock().await;
        if !self.blog_posts_cache.contains_async(&post_id).await {
            self.blog_cache_metrics.record_rejected_admission();
        } else if self.blog_post_slug_cache.contains_async(&normalized).await
            || self.blog_post_slug_cache.len() < BLOG_POST_CACHE_MAX_ENTRIES
        {
            let _ = self.blog_post_slug_cache.upsert_async(normalized, post_id).await;
        } else {
            self.blog_cache_metrics.record_rejected_admission();
        }
        drop(mutation);
    }

    pub async fn get_post_from_cache_by_slug(&self, slug: &str) -> Option<CachedPostInfo> {
        let post_id = self.get_post_id_by_slug_from_cache(slug).await?;
        self.get_post_from_cache(&post_id).await
    }
}
