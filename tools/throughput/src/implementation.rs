//! Build-time identity of the exact harness source and resolved workspace inputs.

pub const fn digest() -> &'static str {
    env!("THROUGHPUT_IMPLEMENTATION_DIGEST")
}

#[cfg(test)]
mod tests {
    #[test]
    fn implementation_digest_is_sha256() {
        let digest = super::digest();
        assert_eq!(digest.len(), 71);
        assert!(digest.starts_with("sha256:"));
        assert!(digest[7..].bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
