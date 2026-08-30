use std::{future::Future, net::IpAddr, pin::Pin, sync::Arc};

use crate::{
    features::geo::service::geo_service::GeoService,
    features::reference_data::service::reference_data_service::ReferenceDataService,
};

pub type FlagFuture<'a> = Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>;

pub trait CountryAlpha2FlagPort: Send + Sync {
    fn flag<'a>(&'a self, code: &'a str) -> FlagFuture<'a>;
}

pub trait GeoIpLookupPort: Send + Sync {
    fn country_alpha2(&self, ip: IpAddr) -> Option<String>;
}

pub struct ReferenceDataAlpha2Flags {
    pub reference_data: Arc<ReferenceDataService>,
}

impl CountryAlpha2FlagPort for ReferenceDataAlpha2Flags {
    fn flag<'a>(&'a self, code: &'a str) -> FlagFuture<'a> {
        Box::pin(async move {
            self.reference_data
                .country_by_alpha2(code)
                .await
                .map(|country| country.country.country_flag)
        })
    }
}

impl GeoIpLookupPort for GeoService {
    fn country_alpha2(&self, ip: IpAddr) -> Option<String> {
        self.lookup(ip).map(|info| info.country_code)
    }
}
