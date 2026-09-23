//! Object keys that an unresolved media-cleanup record's original URL can name.

/// Returns whether `key` in `bucket` is the object named by `original_url`.
///
/// Reconciliation exists for legacy URLs that the strict public-S3 parser rejected, so any
/// `s3`, `https`, or `http` URL is accepted, but only for the key its path spells: the whole
/// path, or the remainder after a leading `<bucket>/` segment for path-style URLs. Both the
/// percent-encoded and decoded spellings match. An `s3` URL must also name the same bucket.
/// Accepting any other key would let one reconciliation delete an unrelated object.
pub(super) fn original_url_names_key(original_url: &str, bucket: &str, key: &str) -> bool {
    let url = match reqwest::Url::parse(original_url) {
        Ok(url) => url,
        Err(_) => return false,
    };
    match url.scheme() {
        "s3" if url.host_str() == Some(bucket) => {}
        "https" | "http" => {}
        _ => return false,
    }
    let encoded = url.path().trim_start_matches('/');
    let decoded = match percent_decode(encoded) {
        Some(decoded) => decoded,
        None => return false,
    };
    [encoded, decoded.as_str()]
        .into_iter()
        .any(|path| path_names_key(path, bucket, key))
}

fn path_names_key(path: &str, bucket: &str, key: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path == key {
        return true;
    }
    match path
        .strip_prefix(bucket)
        .and_then(|rest| rest.strip_prefix('/'))
    {
        Some(path_style_key) => !path_style_key.is_empty() && path_style_key == key,
        None => false,
    }
}

/// Decodes `%XX` escapes into UTF-8, rejecting malformed escapes and invalid UTF-8.
fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                let high = hex_value(*bytes.get(index + 1)?)?;
                let low = hex_value(*bytes.get(index + 2)?)?;
                decoded.push(high << 4 | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::original_url_names_key;

    const BUCKET: &str = "cyhdev-media";

    #[test]
    fn virtual_hosted_and_scheme_urls_name_their_path_key() {
        assert!(original_url_names_key(
            "https://legacy-cdn.example.test/images/a.avif",
            BUCKET,
            "images/a.avif"
        ));
        assert!(original_url_names_key(
            "s3://cyhdev-media/images/a.avif",
            BUCKET,
            "images/a.avif"
        ));
    }

    #[test]
    fn path_style_urls_name_the_key_after_the_bucket_segment() {
        assert!(original_url_names_key(
            "https://s3.us-west-1.amazonaws.com/cyhdev-media/images/a.avif",
            BUCKET,
            "images/a.avif"
        ));
    }

    #[test]
    fn encoded_paths_match_their_decoded_key() {
        assert!(original_url_names_key(
            "https://legacy.example.test/profile%20pictures/%ED%95%9C.avif",
            BUCKET,
            "profile pictures/한.avif"
        ));
    }

    #[test]
    fn unrelated_keys_buckets_and_schemes_are_rejected() {
        let url = "https://legacy-cdn.example.test/images/a.avif";
        assert!(!original_url_names_key(url, BUCKET, "images/b.avif"));
        assert!(!original_url_names_key(url, BUCKET, "a.avif"));
        assert!(!original_url_names_key(url, BUCKET, ""));
        assert!(!original_url_names_key(
            "s3://other-bucket/images/a.avif",
            BUCKET,
            "images/a.avif"
        ));
        assert!(!original_url_names_key(
            "ftp://legacy.example.test/images/a.avif",
            BUCKET,
            "images/a.avif"
        ));
        assert!(!original_url_names_key(
            "not a url",
            BUCKET,
            "images/a.avif"
        ));
        assert!(!original_url_names_key(
            "https://legacy.example.test/bad%zzescape",
            BUCKET,
            "bad%zzescape"
        ));
    }
}
