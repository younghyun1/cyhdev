//! Bounded JSON extraction for blog write endpoints.

use crate::util::extract::bounded_json::{BoundedJson, SOCIAL_JSON_BODY_MAX_BYTES};

/// Post bodies carry up to 500,000 Markdown characters plus JSON escaping.
pub const BLOG_JSON_BODY_MAX_BYTES: usize = 1024 * 1024;

/// Post create and update bodies.
pub type BlogJson<T> = BoundedJson<T, BLOG_JSON_BODY_MAX_BYTES>;

/// Comment and vote bodies, which never need the post-sized allowance.
pub type BlogSocialJson<T> = BoundedJson<T, SOCIAL_JSON_BODY_MAX_BYTES>;
