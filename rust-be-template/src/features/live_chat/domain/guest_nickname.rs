use std::net::IpAddr;

mod words;
pub use words::GUEST_NICKNAME_NOUNS;

// Curated from open English lexical datasets, then frozen into consts for
// zero-runtime I/O nickname generation:
// - WordNet 3.1 adjective satellite/common adjective lemmas.
// - WordNet 3.1 animal noun lemmas, filtered to familiar concrete animals.
pub const GUEST_NICKNAME_ADJECTIVES: &[&str] = &[
    "adventurous",
    "affable",
    "alert",
    "amber",
    "amiable",
    "breezy",
    "bright",
    "brisk",
    "bubbly",
    "calm",
    "carefree",
    "charming",
    "cheerful",
    "clever",
    "cozy",
    "crafty",
    "curious",
    "dapper",
    "dazzling",
    "eager",
    "earnest",
    "easygoing",
    "electric",
    "elegant",
    "fancy",
    "fearless",
    "festive",
    "fluffy",
    "friendly",
    "frosty",
    "gentle",
    "gleeful",
    "gossamer",
    "glimmering",
    "glossy",
    "golden",
    "graceful",
    "happy",
    "hardy",
    "hopeful",
    "helpful",
    "honeyed",
    "jovial",
    "jolly",
    "kind",
    "laughing",
    "lighthearted",
    "lively",
    "lucid",
    "lucky",
    "luminous",
    "merry",
    "mighty",
    "mellow",
    "moonlit",
    "misty",
    "modest",
    "nimble",
    "opal",
    "patient",
    "peaceful",
    "playful",
    "polite",
    "plucky",
    "proud",
    "quiet",
    "quick",
    "radiant",
    "ready",
    "rosy",
    "rugged",
    "sage",
    "sandy",
    "serene",
    "sharp",
    "shiny",
    "silky",
    "sincere",
    "skilled",
    "snappy",
    "soft",
    "sparkly",
    "spirited",
    "sprightly",
    "steady",
    "stellar",
    "swift",
    "sunny",
    "sweet",
    "tender",
    "thoughtful",
    "tidy",
    "tranquil",
    "trusty",
    "upbeat",
    "velvet",
    "vivid",
    "warm",
    "winsome",
    "wise",
    "witty",
    "zesty",
    "zippy",
    "acrobatic",
    "blithe",
    "brave",
    "buoyant",
    "canary",
    "candid",
    "crisp",
    "deft",
    "dreamy",
    "fair",
    "frisky",
    "gallant",
    "hearty",
    "jazzy",
    "keen",
    "loyal",
    "magic",
    "neat",
    "peppy",
    "perky",
    "prime",
    "silvery",
    "sleek",
    "snowy",
    "spry",
    "stout",
    "tidal",
    "twinkly",
    "verdant",
    "whimsical",
    "willowy",
    "zonal",
];

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

pub fn guest_nickname_for_ip(ip: IpAddr) -> String {
    let seed = nickname_seed(ip);
    let adjective = word_from_seed(seed, GUEST_NICKNAME_ADJECTIVES, 0);
    let noun = word_from_seed(seed.rotate_left(32), GUEST_NICKNAME_NOUNS, 1);
    format!("{adjective} {noun}")
}

fn word_from_seed(seed: u64, words: &'static [&'static str], salt: usize) -> &'static str {
    if words.is_empty() {
        return "friendly";
    }
    let mixed = seed ^ ((salt as u64).wrapping_mul(0x9e3779b97f4a7c15));
    let index = (mixed as usize) % words.len();
    words[index]
}

fn nickname_seed(ip: IpAddr) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    match ip {
        IpAddr::V4(ipv4) => {
            hash = write_tagged_octets(hash, 4, &ipv4.octets());
        }
        IpAddr::V6(ipv6) => {
            hash = write_tagged_octets(hash, 6, &ipv6.octets());
        }
    }
    hash
}

fn write_tagged_octets(mut hash: u64, tag: u8, octets: &[u8]) -> u64 {
    hash ^= u64::from(tag);
    hash = hash.wrapping_mul(FNV_PRIME);
    for octet in octets {
        hash ^= u64::from(*octet);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn is_legacy_guest_ip_display_name(display_name: &str, ip: IpAddr) -> bool {
    display_name == format!("guest@{ip}")
}

pub fn normalize_guest_display_name(
    sender_display_name: String,
    sender_kind: i16,
    guest_ip: Option<IpAddr>,
) -> String {
    if sender_kind != super::message::LIVE_CHAT_SENDER_KIND_GUEST {
        return sender_display_name;
    }

    match guest_ip {
        Some(ip) if is_legacy_guest_ip_display_name(&sender_display_name, ip) => {
            guest_nickname_for_ip(ip)
        }
        Some(ip) if sender_display_name.trim().is_empty() => guest_nickname_for_ip(ip),
        _ => sender_display_name,
    }
}

#[cfg(test)]
mod tests;
