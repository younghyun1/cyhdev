use std::collections::HashSet;

use super::*;

#[test]
fn nickname_word_space_is_large_and_unique() {
    assert!(GUEST_NICKNAME_ADJECTIVES.len() >= 120);
    assert!(GUEST_NICKNAME_NOUNS.len() >= 120);
    assert!(GUEST_NICKNAME_ADJECTIVES.len() * GUEST_NICKNAME_NOUNS.len() >= 16_000);
    let mut adjectives = HashSet::new();
    for adjective in GUEST_NICKNAME_ADJECTIVES {
        assert!(
            adjectives.insert(*adjective),
            "duplicate adjective: {adjective}"
        );
    }
    let mut nouns = HashSet::new();
    for noun in GUEST_NICKNAME_NOUNS {
        assert!(nouns.insert(*noun), "duplicate noun: {noun}");
    }
}

#[test]
fn nickname_is_deterministic_for_a_seed() {
    let first = guest_nickname_from_seed(0x0123_4567_89ab_cdef);
    assert_eq!(first, guest_nickname_from_seed(0x0123_4567_89ab_cdef));
    assert_eq!(first.split_whitespace().count(), 2);
    let distinct = (0..64_u64)
        .map(|seed| guest_nickname_from_seed(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15)))
        .collect::<HashSet<_>>();
    assert!(
        distinct.len() > 32,
        "seeds should spread across the word space"
    );
}
