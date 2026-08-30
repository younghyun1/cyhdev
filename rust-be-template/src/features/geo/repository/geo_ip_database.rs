//! Memory-mapped, compressed Geo-IP database storage and lookup.

use std::{collections::BTreeMap, fs::File, net::IpAddr, path::Path};

use crate::features::geo::domain::geo_ip::IpInfo;
use bitcode::Decode;
use internment::Intern;
use memmap2::MmapOptions;

#[derive(Decode, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpRangeKey {
    V4(u32),
    V6(u128),
}

#[derive(Decode)]
struct RawIpEntry {
    start: IpRangeKey,
    end: IpRangeKey,
    country_code: String,
    country_name: String,
    state: String,
    city: String,
    lat: f64,
    lon: f64,
    postal: String,
}

#[derive(Decode)]
struct RawGeoIpBundle {
    entries: BTreeMap<IpRangeKey, RawIpEntry>,
}

pub struct IpEntry {
    end: IpRangeKey,
    country_code: Intern<String>,
    country_name: Intern<String>,
    state: Intern<String>,
    city: Intern<String>,
    postal: Intern<String>,
    latitude: f64,
    longitude: f64,
}

impl IpEntry {
    fn contains(&self, ip: IpAddr) -> bool {
        match (ip, &self.end) {
            (IpAddr::V4(address), IpRangeKey::V4(end)) => u32::from(address) <= *end,
            (IpAddr::V6(address), IpRangeKey::V6(end)) => u128::from(address) <= *end,
            _ => false,
        }
    }
}

pub struct GeoIpDatabases {
    v4: BTreeMap<IpRangeKey, IpEntry>,
    v6: BTreeMap<IpRangeKey, IpEntry>,
}

impl GeoIpDatabases {
    pub fn load_default() -> anyhow::Result<(Self, std::time::Duration)> {
        let start = std::time::Instant::now();
        let v4 = load_database(Path::new("./new_bundle_ipv4.db"))?;
        let v6 = load_database(Path::new("./new_bundle_ipv6.db"))?;
        Ok((Self { v4, v6 }, start.elapsed()))
    }

    pub fn lookup(&self, ip: IpAddr) -> Option<IpInfo> {
        let candidate = match ip {
            IpAddr::V4(address) => self.v4.range(..=IpRangeKey::V4(address.into())).next_back(),
            IpAddr::V6(address) => self.v6.range(..=IpRangeKey::V6(address.into())).next_back(),
        };
        candidate
            .filter(|(_, entry)| entry.contains(ip))
            .map(|(_, entry)| IpInfo {
                ip: ip.to_string(),
                country_code: entry.country_code.to_string(),
                country_name: entry.country_name.to_string(),
                state: entry.state.to_string(),
                city: entry.city.to_string(),
                postal: entry.postal.to_string(),
                latitude: entry.latitude,
                longitude: entry.longitude,
            })
    }
}

fn load_database(path: &Path) -> anyhow::Result<BTreeMap<IpRangeKey, IpEntry>> {
    let file = File::open(path)?;
    // The deployment database files are immutable for the process lifetime;
    // mapping avoids a second compressed-file allocation before zstd decode.
    let mapped = unsafe { MmapOptions::new().map(&file)? };
    let decompressed = zstd::stream::decode_all(&mapped[..])?;
    let raw: RawGeoIpBundle = bitcode::decode(&decompressed)?;
    drop(decompressed);
    Ok(raw
        .entries
        .into_iter()
        .map(|(key, raw)| {
            (
                key,
                IpEntry {
                    end: raw.end,
                    country_code: Intern::new(raw.country_code),
                    country_name: Intern::new(raw.country_name),
                    state: Intern::new(raw.state),
                    city: Intern::new(raw.city),
                    postal: Intern::new(raw.postal),
                    latitude: raw.lat,
                    longitude: raw.lon,
                },
            )
        })
        .collect())
}
