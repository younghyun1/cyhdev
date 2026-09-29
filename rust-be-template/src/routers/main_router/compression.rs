//! Shared response compression follows browser HTTP content-encoding negotiation.

use tower_http::compression::CompressionLayer;

/// Preserve the default predicate: images and already encoded bodies pass through.
pub(super) fn layer() -> CompressionLayer {
    CompressionLayer::new().zstd(true).gzip(true)
}

#[cfg(test)]
#[path = "compression_tests.rs"]
mod tests;
