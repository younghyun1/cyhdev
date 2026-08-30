//! Visitor aggregation and buffered persistence values.

use std::net::IpAddr;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VisitorLogKey {
    pub latitude_bytes: [u8; 8],
    pub longitude_bytes: [u8; 8],
    pub ip_address: IpAddr,
    pub city: String,
    pub country: String,
}

#[derive(Clone)]
pub struct VisitorLogBatch {
    pub count: u64,
    pub visited_at: chrono::DateTime<chrono::Utc>,
}

pub struct NewVisit {
    pub latitude: f64,
    pub longitude: f64,
    pub ip_address: IpAddr,
    pub city: String,
    pub country: String,
    pub visited_at: chrono::DateTime<chrono::Utc>,
}
