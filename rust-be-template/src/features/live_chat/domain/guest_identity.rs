//! Keyed public identity for live-chat guests.
//!
//! Guests are identified server side by IP address, which must never reach a
//! browser. Their public actor key and nickname derive from HMAC-SHA256 over the
//! canonical address under a server secret. An unkeyed hash would not be enough:
//! the IPv4 space is small enough to enumerate, so anyone could reverse it.
//! Both values stay stable for as long as the secret does.

use std::net::IpAddr;

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::guest_nickname::guest_nickname_from_seed;

const HMAC_BLOCK_BYTES: usize = 64;
const HMAC_INNER_PAD: u8 = 0x36;
const HMAC_OUTER_PAD: u8 = 0x5c;
/// Domain separation so the secret can never produce a MAC valid elsewhere.
const GUEST_IDENTITY_DOMAIN: &[u8] = b"cyhdev.live-chat.guest-identity.v1";
/// 128 bits of the MAC become the public key; collisions stay negligible.
const GUEST_ACTOR_KEY_BYTES: usize = 16;
/// Secrets below this length are rejected in favor of a random process secret.
pub const GUEST_IDENTITY_MIN_SECRET_BYTES: usize = 32;
const RANDOM_SECRET_BYTES: usize = 32;

/// Public identity derived for one guest address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuestIdentity {
    /// Lowercase hex, safe inside WebRTC stream ids and JSON.
    pub actor_key: String,
    pub nickname: String,
}

/// HMAC-SHA256 key (RFC 2104) held as the zero-padded block it expands to.
pub struct GuestIdentityKey {
    block: Zeroizing<[u8; HMAC_BLOCK_BYTES]>,
}

impl GuestIdentityKey {
    /// Build a key from configured secret bytes. Secrets longer than the hash
    /// block are hashed first, as RFC 2104 requires.
    pub fn from_secret(secret: &[u8]) -> Self {
        let mut block = Zeroizing::new([0_u8; HMAC_BLOCK_BYTES]);
        if secret.len() > HMAC_BLOCK_BYTES {
            let digest: [u8; 32] = Sha256::digest(secret).into();
            block[..digest.len()].copy_from_slice(&digest);
        } else {
            block[..secret.len()].copy_from_slice(secret);
        }
        Self { block }
    }

    /// Build a key from operating-system entropy; identities change on restart.
    pub fn random() -> Result<Self, getrandom::Error> {
        let mut secret = Zeroizing::new([0_u8; RANDOM_SECRET_BYTES]);
        getrandom::fill(secret.as_mut())?;
        Ok(Self::from_secret(secret.as_ref()))
    }

    /// Derive the public key and nickname for a guest address.
    pub fn identify(&self, ip: IpAddr) -> GuestIdentity {
        let (family, octets) = match ip.to_canonical() {
            IpAddr::V4(ip) => (4_u8, ip.to_ipv6_mapped().octets()),
            IpAddr::V6(ip) => (6_u8, ip.octets()),
        };
        let mac = self.mac(&[GUEST_IDENTITY_DOMAIN, &[family], &octets]);
        let mut seed = [0_u8; 8];
        seed.copy_from_slice(&mac[GUEST_ACTOR_KEY_BYTES..GUEST_ACTOR_KEY_BYTES + 8]);
        GuestIdentity {
            actor_key: lower_hex(&mac[..GUEST_ACTOR_KEY_BYTES]),
            nickname: guest_nickname_from_seed(u64::from_be_bytes(seed)),
        }
    }

    fn mac(&self, message: &[&[u8]]) -> [u8; 32] {
        let mut inner_pad = Zeroizing::new([HMAC_INNER_PAD; HMAC_BLOCK_BYTES]);
        let mut outer_pad = Zeroizing::new([HMAC_OUTER_PAD; HMAC_BLOCK_BYTES]);
        for (index, byte) in self.block.iter().enumerate() {
            inner_pad[index] ^= byte;
            outer_pad[index] ^= byte;
        }
        let mut inner = Sha256::new();
        inner.update(inner_pad.as_ref());
        for part in message {
            inner.update(part);
        }
        let inner_digest: [u8; 32] = inner.finalize().into();
        let mut outer = Sha256::new();
        outer.update(outer_pad.as_ref());
        outer.update(inner_digest);
        outer.finalize().into()
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    fn mac_hex(key: &[u8], message: &[u8]) -> String {
        lower_hex(&GuestIdentityKey::from_secret(key).mac(&[message]))
    }

    #[test]
    fn hmac_matches_rfc_4231_vectors() {
        assert_eq!(
            mac_hex(&[0x0b; 20], b"Hi There"),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            mac_hex(b"Jefe", b"what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            mac_hex(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            ),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn identity_is_stable_per_secret_and_hides_the_address() {
        let key = GuestIdentityKey::from_secret(&[7; 32]);
        let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7));
        let identity = key.identify(ip);
        assert_eq!(identity, key.identify(ip));
        assert_eq!(identity.actor_key.len(), GUEST_ACTOR_KEY_BYTES * 2);
        assert!(!identity.actor_key.contains("203"));
        assert!(!identity.nickname.contains("203.0.113.7"));
        assert_eq!(identity.nickname.split_whitespace().count(), 2);
        let other = GuestIdentityKey::from_secret(&[8; 32]).identify(ip);
        assert_ne!(identity.actor_key, other.actor_key);
    }

    #[test]
    fn ipv4_mapped_addresses_share_the_ipv4_identity() {
        let key = GuestIdentityKey::from_secret(&[9; 32]);
        let ip = Ipv4Addr::new(198, 51, 100, 42);
        assert_eq!(
            key.identify(IpAddr::V4(ip)),
            key.identify(IpAddr::V6(ip.to_ipv6_mapped()))
        );
        assert_ne!(
            key.identify(IpAddr::V4(ip)),
            key.identify(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)))
        );
    }
}
