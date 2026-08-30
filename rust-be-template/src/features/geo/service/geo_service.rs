//! In-memory Geo-IP lookup and country presentation service.

use std::{net::IpAddr, sync::Arc};

use uuid::Uuid;

use crate::features::geo::{
    domain::geo_ip::IpInfo,
    repository::{geo_ip_database::GeoIpDatabases, geo_repository::GeoRepository},
};
use crate::features::reference_data::service::reference_data_service::ReferenceDataService;

#[async_trait::async_trait]
pub trait GeoCountryFlagPort: Send + Sync {
    async fn flag_by_numeric_code(&self, country_code: i32) -> Option<String>;
    async fn flag_by_alpha2(&self, country_alpha2: &str) -> Option<String>;
}

#[async_trait::async_trait]
impl GeoCountryFlagPort for ReferenceDataService {
    async fn flag_by_numeric_code(&self, country_code: i32) -> Option<String> {
        self.country_flag(country_code).await
    }

    async fn flag_by_alpha2(&self, country_alpha2: &str) -> Option<String> {
        self.country_by_alpha2(country_alpha2)
            .await
            .map(|country| country.country.country_flag)
    }
}

pub struct GeoService {
    databases: GeoIpDatabases,
    country_flags: Arc<dyn GeoCountryFlagPort>,
    repository: Arc<GeoRepository>,
}

impl GeoService {
    pub fn new(
        databases: GeoIpDatabases,
        country_flags: Arc<dyn GeoCountryFlagPort>,
        repository: Arc<GeoRepository>,
    ) -> Self {
        Self { databases, country_flags, repository }
    }

    pub fn lookup(&self, ip: IpAddr) -> Option<IpInfo> {
        self.databases.lookup(ip)
    }

    pub async fn country_flag(&self, country_code: i32) -> Option<String> {
        self.country_flags.flag_by_numeric_code(country_code).await
    }

    pub async fn country_flag_for_ip(&self, ip: IpAddr) -> Option<String> {
        let info = self.lookup(ip)?;
        self.country_flags.flag_by_alpha2(&info.country_code).await
    }

    pub async fn active_profile_picture(&self, user_id: Uuid) -> Option<String> {
        match self.repository.active_profile_picture(user_id).await {
            Ok(url) => url,
            Err(error) => {
                tracing::error!(error = %error, "Failed to query active profile picture");
                None
            }
        }
    }
}
