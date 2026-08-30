use std::{collections::HashSet, net::{IpAddr, Ipv4Addr, Ipv6Addr}};

use super::*;

#[test]
fn nickname_word_space_is_large_and_unique() {
    assert!(GUEST_NICKNAME_ADJECTIVES.len() >= 120);
    assert!(GUEST_NICKNAME_NOUNS.len() >= 120);
    assert!(GUEST_NICKNAME_ADJECTIVES.len() * GUEST_NICKNAME_NOUNS.len() >= 16_000);
    let mut adjectives = HashSet::new();
    for adjective in GUEST_NICKNAME_ADJECTIVES {
        assert!(adjectives.insert(*adjective), "duplicate adjective: {adjective}");
    }
    let mut nouns = HashSet::new();
    for noun in GUEST_NICKNAME_NOUNS {
        assert!(nouns.insert(*noun), "duplicate noun: {noun}");
    }
}

#[test]
fn nickname_is_deterministic_for_ipv4() {
    let ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7));
    let first = guest_nickname_for_ip(ip);
    assert_eq!(first, guest_nickname_for_ip(ip));
    assert!(!first.contains("203.0.113.7"));
    assert_eq!(first.split_whitespace().count(), 2);
}

#[test]
fn nickname_handles_ipv6_without_leaking_address() {
    let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1));
    let nickname = guest_nickname_for_ip(ip);
    assert!(!nickname.contains("2001"));
    assert_eq!(nickname.split_whitespace().count(), 2);
}

#[test]
fn normalizes_legacy_guest_and_keeps_user_names() {
    let ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 42));
    let guest = normalize_guest_display_name(
        "guest@198.51.100.42".to_owned(),
        super::super::message::LIVE_CHAT_SENDER_KIND_GUEST,
        Some(ip),
    );
    assert_eq!(guest, guest_nickname_for_ip(ip));
    let user = normalize_guest_display_name(
        "younghyun".to_owned(),
        super::super::message::LIVE_CHAT_SENDER_KIND_USER,
        None,
    );
    assert_eq!(user, "younghyun");
}
