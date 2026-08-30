use std::sync::Arc;

use crate::features::reference_data::service::reference_data_service::CountryFlagLookupPort;
use super::rtc::coordinator::RtcCoordinator;
use super::ports::{CountryAlpha2FlagPort, GeoIpLookupPort};
use super::{cache::LiveChatCache, super::repository::live_chat_repository::LiveChatRepository};

pub struct LiveChatService {
    pub(super) repository: Arc<LiveChatRepository>,
    pub cache: Arc<LiveChatCache>,
    pub(super) country_flags: Arc<dyn CountryFlagLookupPort>,
    pub(super) alpha2_flags: Arc<dyn CountryAlpha2FlagPort>,
    pub(super) geo_ip: Arc<dyn GeoIpLookupPort>,
    pub rtc: Arc<RtcCoordinator>,
}

impl LiveChatService {
    pub fn new(
        repository: Arc<LiveChatRepository>,
        cache: Arc<LiveChatCache>,
        country_flags: Arc<dyn CountryFlagLookupPort>,
        alpha2_flags: Arc<dyn CountryAlpha2FlagPort>,
        geo_ip: Arc<dyn GeoIpLookupPort>,
        rtc: Arc<RtcCoordinator>,
    ) -> Self {
        Self { repository, cache, country_flags, alpha2_flags, geo_ip, rtc }
    }

    pub fn rtc(&self) -> Arc<RtcCoordinator> { Arc::clone(&self.rtc) }
}
