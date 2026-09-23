//! Bounded JSON bodies for photograph comment and vote writes.

use crate::util::extract::bounded_json::{BoundedJson, SOCIAL_JSON_BODY_MAX_BYTES};

/// Comment and vote bodies are small; the router default is sized for uploads.
pub(super) type SocialJson<T> = BoundedJson<T, SOCIAL_JSON_BODY_MAX_BYTES>;
