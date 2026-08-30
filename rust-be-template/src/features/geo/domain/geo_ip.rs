//! Persistence-free Geo-IP lookup values.

use utoipa::ToSchema;

#[derive(serde::Serialize, Clone, ToSchema)]
pub struct IpInfo {
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub state: String,
    pub city: String,
    pub postal: String,
    pub latitude: f64,
    pub longitude: f64,
}
