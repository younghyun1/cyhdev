//! Admission policy for ICE candidates supplied by browsers.
//!
//! Every remote candidate makes the SFU send STUN connectivity checks to the
//! named address, so an unfiltered candidate turns the server into a UDP
//! prober of its own network (loopback, private ranges, link-local, cloud
//! metadata) or of arbitrary multicast groups.
//!
//! Filtering does not break calls. Browsers get no ICE servers, so they
//! gather host candidates only: private LAN addresses, mDNS `.local` names,
//! or global IPv6 addresses. The SFU advertises its public address as a host
//! candidate, the browser (the ICE controlling agent) sends checks to it, and
//! the SFU learns the browser's NAT-mapped address as a peer-reflexive
//! candidate from those authenticated checks. That path never consults the
//! candidates filtered here, which only add probes from the SFU side.
//!
//! mDNS names are always dropped. Resolving them would send multicast
//! queries on the server's own LAN, which can never find a remote browser;
//! in local development the browser still reaches the loopback or LAN SFU
//! candidate directly and connectivity again arrives peer-reflexively.
//! Private and loopback addresses are admitted only when the SFU itself is
//! configured on such an address, since browser and SFU then share a network.

use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// Remote candidates admitted per peer connection, trickled or in SDP.
/// Browsers produce a handful per network interface.
pub const MAX_REMOTE_CANDIDATES_PER_PEER: usize = 32;

const SDP_CANDIDATE_PREFIX: &str = "a=candidate:";
const TRICKLE_CANDIDATE_PREFIX: &str = "candidate:";
/// `foundation component transport priority address port typ type`.
const CANDIDATE_ADDRESS_FIELD: usize = 4;
const CANDIDATE_MIN_FIELDS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AddressScope {
    Public,
    /// Loopback, RFC 1918, CGNAT, link-local, and unique-local addresses.
    Local,
    /// Unspecified, multicast, broadcast, documentation, and reserved ranges.
    Unroutable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteCandidatePolicy {
    allow_local: bool,
}

impl RemoteCandidatePolicy {
    /// Derive the policy from the address the SFU advertises (`RTC_PUBLIC_IP`).
    /// An unparsable value keeps the strict public-only policy.
    pub fn for_advertised_address(advertised: &str) -> Self {
        let allow_local = match advertised.trim().parse::<IpAddr>() {
            Ok(ip) => address_scope(ip) == AddressScope::Local,
            Err(_) => false,
        };
        Self { allow_local }
    }

    /// Whether a trickled candidate string may reach the ICE agent. The empty
    /// end-of-candidates marker is dropped; the SFU never waits for it.
    pub fn admits_candidate(&self, candidate: &str) -> bool {
        let attribute = candidate
            .trim()
            .strip_prefix(TRICKLE_CANDIDATE_PREFIX)
            .unwrap_or(candidate.trim());
        self.admits_attribute(attribute)
    }

    /// Remove inadmissible `a=candidate:` lines from a browser SDP and cap
    /// the rest. Browsers include already-gathered candidates in offers and
    /// answers, so SDP is a second path into the ICE agent.
    pub fn sanitize_remote_sdp<'a>(&self, sdp: &'a str) -> Cow<'a, str> {
        if !sdp.contains(SDP_CANDIDATE_PREFIX) {
            return Cow::Borrowed(sdp);
        }
        let mut kept = 0usize;
        let mut sanitized = String::with_capacity(sdp.len());
        for line in sdp.split_inclusive('\n') {
            let content = line.trim_end_matches(['\r', '\n']);
            if let Some(attribute) = content.strip_prefix(SDP_CANDIDATE_PREFIX) {
                if kept >= MAX_REMOTE_CANDIDATES_PER_PEER || !self.admits_attribute(attribute) {
                    continue;
                }
                kept += 1;
            }
            sanitized.push_str(line);
        }
        Cow::Owned(sanitized)
    }

    fn admits_attribute(&self, attribute: &str) -> bool {
        let fields = attribute.split_ascii_whitespace().collect::<Vec<_>>();
        if fields.len() < CANDIDATE_MIN_FIELDS {
            return false;
        }
        // Hostnames, including mDNS `.local` names, would need resolution.
        let Ok(ip) = fields[CANDIDATE_ADDRESS_FIELD].parse::<IpAddr>() else {
            return false;
        };
        match address_scope(ip) {
            AddressScope::Public => true,
            AddressScope::Local => self.allow_local,
            AddressScope::Unroutable => false,
        }
    }
}

fn address_scope(ip: IpAddr) -> AddressScope {
    match ip.to_canonical() {
        IpAddr::V4(ip) => ipv4_scope(ip),
        IpAddr::V6(ip) => ipv6_scope(ip),
    }
}

fn ipv4_scope(ip: Ipv4Addr) -> AddressScope {
    let [first, second, ..] = ip.octets();
    let shared_cgnat = first == 100 && (second & 0xc0) == 64;
    let this_network = first == 0;
    let benchmarking = first == 198 && (second & 0xfe) == 18;
    let reserved = first >= 240;
    if ip.is_unspecified()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_documentation()
        || this_network
        || benchmarking
        || reserved
    {
        AddressScope::Unroutable
    } else if ip.is_loopback() || ip.is_private() || ip.is_link_local() || shared_cgnat {
        AddressScope::Local
    } else {
        AddressScope::Public
    }
}

fn ipv6_scope(ip: Ipv6Addr) -> AddressScope {
    let segments = ip.segments();
    let documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;
    let deprecated_site_local = (segments[0] & 0xffc0) == 0xfec0;
    if ip.is_unspecified() || ip.is_multicast() || documentation {
        AddressScope::Unroutable
    } else if ip.is_loopback()
        || ip.is_unique_local()
        || ip.is_unicast_link_local()
        || deprecated_site_local
    {
        AddressScope::Local
    } else {
        AddressScope::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLIC: RemoteCandidatePolicy = RemoteCandidatePolicy { allow_local: false };
    const LOCAL: RemoteCandidatePolicy = RemoteCandidatePolicy { allow_local: true };

    fn host(address: &str) -> String {
        format!("candidate:842163049 1 udp 2122260223 {address} 54321 typ host generation 0")
    }

    #[test]
    fn public_sfu_admits_only_globally_routable_addresses() {
        for address in ["8.8.4.4", "2606:4700::1111", "::ffff:8.8.4.4"] {
            assert!(PUBLIC.admits_candidate(&host(address)), "{address}");
        }
        for address in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.9",
            "192.168.1.20",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "224.0.0.251",
            "255.255.255.255",
            "192.0.2.10",
            "::1",
            "fe80::1",
            "fd00::1",
            "ff02::fb",
            "::",
            "2001:db8::1",
            "::ffff:10.0.0.1",
            "4f0c7a1e-3b8d-4c9e.local",
            "example.com",
        ] {
            assert!(!PUBLIC.admits_candidate(&host(address)), "{address}");
        }
        assert!(!PUBLIC.admits_candidate(""));
        assert!(!PUBLIC.admits_candidate("candidate:1 1 udp 1 8.8.4.4"));
    }

    #[test]
    fn local_sfu_admits_its_own_networks_but_not_mdns_or_multicast() {
        assert_eq!(
            RemoteCandidatePolicy::for_advertised_address("127.0.0.1"),
            LOCAL
        );
        assert_eq!(
            RemoteCandidatePolicy::for_advertised_address("192.168.0.10"),
            LOCAL
        );
        assert_eq!(
            RemoteCandidatePolicy::for_advertised_address("203.0.113.7"),
            PUBLIC
        );
        assert_eq!(
            RemoteCandidatePolicy::for_advertised_address("sfu.example"),
            PUBLIC
        );
        for address in ["127.0.0.1", "192.168.1.20", "fe80::1", "8.8.4.4"] {
            assert!(LOCAL.admits_candidate(&host(address)), "{address}");
        }
        for address in ["abcd.local", "224.0.0.251", "0.0.0.0"] {
            assert!(!LOCAL.admits_candidate(&host(address)), "{address}");
        }
    }

    #[test]
    fn sdp_candidates_are_filtered_and_capped() {
        let mut sdp = String::from("v=0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\n");
        sdp.push_str(&format!("a={}\r\n", host("10.0.0.2")));
        sdp.push_str(&format!("a={}\r\n", host("abcd.local")));
        for index in 0..40 {
            sdp.push_str(&format!("a={}\r\n", host(&format!("8.8.{index}.1"))));
        }
        sdp.push_str("a=end-of-candidates\r\n");
        let sanitized = PUBLIC.sanitize_remote_sdp(&sdp);
        assert!(!sanitized.contains("10.0.0.2"));
        assert!(!sanitized.contains(".local"));
        assert_eq!(
            sanitized.matches("a=candidate:").count(),
            MAX_REMOTE_CANDIDATES_PER_PEER
        );
        assert!(sanitized.starts_with("v=0\r\nm=audio"));
        assert!(sanitized.ends_with("a=end-of-candidates\r\n"));
        let untouched = "v=0\r\ns=-\r\n";
        assert!(matches!(
            PUBLIC.sanitize_remote_sdp(untouched),
            Cow::Borrowed(_)
        ));
    }
}
