use super::super::repository::photography_repository::PhotographyRepository;
use super::batch::BatchRegistry;
use super::media::MediaPorts;
use super::views::photograph_view_buffer;
use crate::features::accounts::service::account_service::AccountService;
use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;
use crate::util::media::object_store::MediaObjectStore;
use crate::util::view_delta_buffer::ViewDeltaBuffer;
use std::sync::Arc;

pub struct PhotographyService {
    pub(super) repository: PhotographyRepository,
    pub(super) views: ViewDeltaBuffer,
    pub(super) batches: BatchRegistry,
    pub(super) media: MediaPorts,
    pub(super) flags: Arc<dyn CountryFlagLookupPort>,
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
        }
    }
}
