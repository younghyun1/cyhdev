use std::sync::Arc;

use super::guest_identity::load_guest_identity_key;
use super::ports::{CountryAlpha2FlagPort, GeoIpLookupPort};
use super::rtc::coordinator::RtcCoordinator;
use super::{super::repository::live_chat_repository::LiveChatRepository, cache::LiveChatCache};
use crate::features::live_chat::{domain::guest_identity::GuestIdentityKey, error::LiveChatError};
use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;

pub struct LiveChatService {
    pub(super) repository: Arc<LiveChatRepository>,
    pub cache: Arc<LiveChatCache>,
    pub(super) country_flags: Arc<dyn CountryFlagLookupPort>,
    pub(super) alpha2_flags: Arc<dyn CountryAlpha2FlagPort>,
    pub(super) geo_ip: Arc<dyn GeoIpLookupPort>,
    pub(super) guest_identity: GuestIdentityKey,
    pub rtc: Arc<RtcCoordinator>,
}

impl LiveChatService {
    /// Compose the service, loading the guest identity secret from the
    /// environment. Fails only when no secret is configured and the operating
    /// system cannot supply entropy for a random one.
    pub fn new(
        repository: Arc<LiveChatRepository>,
        cache: Arc<LiveChatCache>,
        country_flags: Arc<dyn CountryFlagLookupPort>,
        alpha2_flags: Arc<dyn CountryAlpha2FlagPort>,
        geo_ip: Arc<dyn GeoIpLookupPort>,
        rtc: Arc<RtcCoordinator>,
    ) -> Result<Self, LiveChatError> {
        let guest_identity = load_guest_identity_key().map_err(LiveChatError::Entropy)?;
        Ok(Self {
            repository,
            cache,
            country_flags,
            alpha2_flags,
            geo_ip,
            guest_identity,
            rtc,
        })
    }

    pub fn rtc(&self) -> Arc<RtcCoordinator> {
        Arc::clone(&self.rtc)
    }
}
