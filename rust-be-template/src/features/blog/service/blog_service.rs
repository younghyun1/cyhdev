use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::features::accounts::domain::public_author::PublicAuthor;
use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;

use super::super::{domain::cache::CachedPostInfo, repository::blog_repository::BlogRepository};
use super::{cache_policy::BlogCacheMetrics, search::search_index::PostSearchIndex};

const BLOG_POST_USE_CASE_STRIPES: usize = 64;

pub struct BlogService {
    pub(super) repository: Arc<BlogRepository>,
    pub(super) posts_cache: scc::HashMap<Uuid, CachedPostInfo>,
    pub(super) slug_cache: scc::HashMap<String, Uuid>,
    pub(super) order_cache: RwLock<Vec<Uuid>>,
    pub(super) cache_mutation: Mutex<()>,
    post_use_cases: [Mutex<()>; BLOG_POST_USE_CASE_STRIPES],
    pub(super) metrics: BlogCacheMetrics,
    pub(super) search_index: Arc<PostSearchIndex>,
    pub(super) country_flags: Arc<dyn CountryFlagLookupPort>,
}

impl BlogService {
    pub fn new(
        repository: Arc<BlogRepository>,
        search_index: Arc<PostSearchIndex>,
        country_flags: Arc<dyn CountryFlagLookupPort>,
    ) -> Self {
        Self {
            repository,
            posts_cache: scc::HashMap::new(),
            slug_cache: scc::HashMap::new(),
            order_cache: RwLock::new(Vec::new()),
            cache_mutation: Mutex::new(()),
            post_use_cases: std::array::from_fn(|_| Mutex::new(())),
            metrics: BlogCacheMetrics::default(),
            search_index,
            country_flags,
        }
    }

    pub(super) async fn country_flags_for_authors(
        &self,
        authors: &HashMap<Uuid, PublicAuthor>,
    ) -> HashMap<i32, String> {
        let mut country_codes = authors
            .values()
            .filter_map(PublicAuthor::country_code)
            .collect::<Vec<_>>();
        country_codes.sort_unstable();
        country_codes.dedup();
        self.country_flags.country_flags(&country_codes).await
    }

    /// Serializes one post's database mutation through cache and search publication.
    pub(super) async fn lock_post_use_case(
        &self,
        post_id: Uuid,
    ) -> tokio::sync::MutexGuard<'_, ()> {
        self.post_use_cases[post_stripe(post_id)].lock().await
    }

    /// Locks all stripes touched by a bounded hydration batch in stable order.
    pub(super) async fn lock_post_set(
        &self,
        post_ids: &[Uuid],
    ) -> Vec<tokio::sync::MutexGuard<'_, ()>> {
        let mut stripes = post_ids
            .iter()
            .map(|post_id| post_stripe(*post_id))
            .collect::<Vec<_>>();
        stripes.sort_unstable();
        stripes.dedup();
        let mut guards = Vec::with_capacity(stripes.len());
        for stripe in stripes {
            guards.push(self.post_use_cases[stripe].lock().await);
        }
        guards
    }
}

fn post_stripe(post_id: Uuid) -> usize {
    usize::from(post_id.as_bytes()[15]) % BLOG_POST_USE_CASE_STRIPES
}

#[cfg(test)]
mod tests {
    use super::{BLOG_POST_USE_CASE_STRIPES, post_stripe};
    use uuid::Uuid;

    #[test]
    fn post_stripes_are_fixed_and_bounded() {
        for suffix in u8::MIN..=u8::MAX {
            let mut bytes = [0_u8; 16];
            bytes[15] = suffix;
            assert!(post_stripe(Uuid::from_bytes(bytes)) < BLOG_POST_USE_CASE_STRIPES);
        }
    }
}
