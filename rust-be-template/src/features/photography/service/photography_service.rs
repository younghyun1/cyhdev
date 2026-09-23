use super::super::repository::photography_repository::PhotographyRepository;
use super::batch::BatchRegistry;
use super::media::MediaPorts;
use super::views::photograph_view_buffer;
use crate::features::accounts::service::account_service::AccountService;
use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;
use crate::util::media::object_store::MediaObjectStore;
use crate::util::{
    actor_write_limiter::{ActorWriteLimiter, WriteBudget},
    view_delta_buffer::ViewDeltaBuffer,
};
use std::{sync::Arc, time::Duration};

use super::super::error::PhotographyError;
use uuid::Uuid;

/// Accounts tracked by the comment and vote write budget.
pub const MAX_PHOTOGRAPH_WRITE_ACTORS: usize = 16_384;
const PHOTOGRAPH_WRITE_WINDOW: Duration = Duration::from_secs(10 * 60);
/// Comment creates, edits, and deletions per account per window.
const PHOTOGRAPH_COMMENT_WRITES: u32 = 30;
/// Photograph and comment votes and withdrawals per account per window.
const PHOTOGRAPH_VOTE_WRITES: u32 = 120;

/// Independent write budgets charged before a comment or vote transaction.
#[derive(Clone, Copy)]
pub(super) enum PhotographWriteKind {
    Comment,
    Vote,
}

impl PhotographWriteKind {
    const fn index(self) -> usize {
        match self {
            Self::Comment => 0,
            Self::Vote => 1,
        }
    }
}

pub struct PhotographyService {
    pub(super) repository: PhotographyRepository,
    pub(super) views: ViewDeltaBuffer,
    pub(super) batches: BatchRegistry,
    pub(super) media: MediaPorts,
    pub(super) flags: Arc<dyn CountryFlagLookupPort>,
    write_limiter: ActorWriteLimiter<2>,
}

impl PhotographyService {
    pub fn new(
        repository: PhotographyRepository,
        object_store: Arc<dyn MediaObjectStore>,
        object_store_region: Arc<str>,
        accounts: Arc<AccountService>,
        flags: Arc<dyn CountryFlagLookupPort>,
    ) -> Self {
        Self {
            repository,
            views: photograph_view_buffer(),
            batches: BatchRegistry::new(),
            media: MediaPorts::new(object_store, object_store_region, accounts),
            flags,
            write_limiter: ActorWriteLimiter::new(
                MAX_PHOTOGRAPH_WRITE_ACTORS,
                [
                    WriteBudget {
                        attempts: PHOTOGRAPH_COMMENT_WRITES,
                        window: PHOTOGRAPH_WRITE_WINDOW,
                    },
                    WriteBudget {
                        attempts: PHOTOGRAPH_VOTE_WRITES,
                        window: PHOTOGRAPH_WRITE_WINDOW,
                    },
                ],
            ),
        }
    }

    /// Charges one attempt against the account's comment or vote budget.
    pub(super) async fn charge_write(
        &self,
        user_id: Uuid,
        kind: PhotographWriteKind,
    ) -> Result<(), PhotographyError> {
        self.write_limiter
            .check(user_id, kind.index())
            .await
            .map_err(|rejection| PhotographyError::WriteThrottled {
                retry_after: rejection.retry_after,
                saturated: rejection.saturated,
            })
    }
}
