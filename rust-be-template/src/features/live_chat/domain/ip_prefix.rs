//! Client address grouping for live-chat rate limits, connection caps, and bans.
//!
//! IPv4 addresses stay exact. IPv6 addresses group by their /64 because one
//! subscriber normally controls a whole /64 and can rotate through it freely, so
//! exact-address keys would let a single host fill every bounded table.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnet::{IpNet, Ipv4Net, Ipv6Net};

/// Prefix length shared by one IPv6 subscriber.
pub const IPV6_SUBSCRIBER_PREFIX_LEN: u8 = 64;

/// Canonical address group used as a rate, connection, and ban key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LiveChatIpPrefix {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl LiveChatIpPrefix {
    /// Group an address, first mapping IPv4-mapped IPv6 back to IPv4 so a
    /// dual-stack listener cannot give one client two identities.
    pub fn of(ip: IpAddr) -> Self {
        match ip.to_canonical() {
            IpAddr::V4(ip) => Self::V4(ip),
            IpAddr::V6(ip) => Self::V6(Ipv6Addr::from(
                u128::from(ip) & (u128::MAX << (128 - u32::from(IPV6_SUBSCRIBER_PREFIX_LEN))),
            )),
        }
    }

    /// Network stored for a ban covering this group.
    pub fn network(self) -> IpNet {
        match self {
            Self::V4(ip) => IpNet::V4(Ipv4Net::from(ip)),
            Self::V6(ip) => match Ipv6Net::new(ip, IPV6_SUBSCRIBER_PREFIX_LEN) {
                Ok(network) => IpNet::V6(network),
                // Unreachable for the constant prefix length; fall back to the
                // narrower host network rather than widening the ban.
                Err(_) => IpNet::V6(Ipv6Net::from(ip)),
            },
        }
    }
}

/// Networks whose ban applies to `ip`: its group network and, for IPv6, the
/// exact host network recorded by bans created before /64 grouping.
pub fn ban_networks_for(ip: IpAddr) -> BanNetworks {
    let canonical = ip.to_canonical();
    let group = LiveChatIpPrefix::of(canonical).network();
    let host = IpNet::from(canonical);
    BanNetworks {
        group,
        host: (host != group).then_some(host),
    }
}

/// At most two networks, kept inline so every ban lookup stays allocation free.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BanNetworks {
    group: IpNet,
    host: Option<IpNet>,
}

impl BanNetworks {
    pub fn iter(&self) -> impl Iterator<Item = IpNet> + '_ {
        std::iter::once(self.group).chain(self.host)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn ip(value: &str) -> IpAddr {
        match IpAddr::from_str(value) {
            Ok(ip) => ip,
            Err(error) => panic!("valid IP expected: {error}"),
        }
    }

    #[test]
    fn ipv4_stays_exact_and_mapped_addresses_normalize() {
        assert_eq!(
            LiveChatIpPrefix::of(ip("203.0.113.9")),
            LiveChatIpPrefix::of(ip("::ffff:203.0.113.9"))
        );
        assert_ne!(
            LiveChatIpPrefix::of(ip("203.0.113.9")),
            LiveChatIpPrefix::of(ip("203.0.113.10"))
        );
        assert_eq!(
            LiveChatIpPrefix::of(ip("203.0.113.9"))
                .network()
                .to_string(),
            "203.0.113.9/32"
        );
    }

    #[test]
    fn ipv6_groups_by_subscriber_prefix() {
        let first = LiveChatIpPrefix::of(ip("2001:db8:1:2:aaaa::1"));
        assert_eq!(first, LiveChatIpPrefix::of(ip("2001:db8:1:2:ffff::9")));
        assert_ne!(first, LiveChatIpPrefix::of(ip("2001:db8:1:3::1")));
        assert_eq!(first.network().to_string(), "2001:db8:1:2::/64");
    }

    #[test]
    fn ban_networks_cover_group_and_legacy_host_entries() {
        let v6: Vec<String> = ban_networks_for(ip("2001:db8::5"))
            .iter()
            .map(|network| network.to_string())
            .collect();
        assert_eq!(v6, ["2001:db8::/64", "2001:db8::5/128"]);
        let v4: Vec<String> = ban_networks_for(ip("::ffff:198.51.100.7"))
            .iter()
            .map(|network| network.to_string())
            .collect();
        assert_eq!(v4, ["198.51.100.7/32"]);
    }
}
