use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::features::accounts::domain::public_author::PublicAuthor;
use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;
use crate::util::{
    actor_write_limiter::{ActorWriteLimiter, WriteBudget},
    view_delta_buffer::ViewDeltaBuffer,
};

use super::super::{
    domain::cache::CachedPostInfo, error::BlogError, repository::blog_repository::BlogRepository,
};
use super::{cache_policy::BlogCacheMetrics, search::search_index::PostSearchIndex};

const BLOG_POST_USE_CASE_STRIPES: usize = 64;
/// Distinct posts whose views may wait in memory between flushes.
pub const BLOG_VIEW_BUFFER_MAX_ENTRIES: usize = 8_192;
/// Accounts tracked by the comment and vote write budget.
pub const MAX_BLOG_WRITE_ACTORS: usize = 16_384;
const BLOG_WRITE_WINDOW: std::time::Duration = std::time::Duration::from_secs(10 * 60);
/// Comment creates, edits, and deletions per account per window.
const BLOG_COMMENT_WRITES: u32 = 30;
/// Post and comment votes and withdrawals per account per window.
const BLOG_VOTE_WRITES: u32 = 120;

/// Independent write budgets charged before a comment or vote transaction.
#[derive(Clone, Copy)]
pub(super) enum BlogWriteKind {
    Comment,
    Vote,
}

impl BlogWriteKind {
    const fn index(self) -> usize {
        match self {
            Self::Comment => 0,
            Self::Vote => 1,
        }
    }
}

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
    pub(super) views: ViewDeltaBuffer,
    write_limiter: ActorWriteLimiter<2>,
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
            views: ViewDeltaBuffer::new("blog_post", BLOG_VIEW_BUFFER_MAX_ENTRIES),
            write_limiter: ActorWriteLimiter::new(
                MAX_BLOG_WRITE_ACTORS,
                [
                    WriteBudget {
                        attempts: BLOG_COMMENT_WRITES,
                        window: BLOG_WRITE_WINDOW,
                    },
                    WriteBudget {
                        attempts: BLOG_VOTE_WRITES,
                        window: BLOG_WRITE_WINDOW,
                    },
                ],
            ),
        }
    }

    /// Charges one attempt against the account's comment or vote budget.
    pub(super) async fn charge_write(
        &self,
        user_id: Uuid,
        kind: BlogWriteKind,
    ) -> Result<(), BlogError> {
        self.write_limiter
            .check(user_id, kind.index())
            .await
            .map_err(|rejection| BlogError::WriteThrottled {
                retry_after: rejection.retry_after,
                saturated: rejection.saturated,
            })
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
