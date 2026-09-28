//! Bounded, read-only Java 26.3 biome predictions using pinned Pumpkin code.

mod generator;
mod request;
mod wire;

pub use generator::{GENERATOR_REVISION, predict};
pub use request::{Cell, PredictionRequest, PredictionResponse};
pub use wire::run;

/// Maximum serialized request size, including its final newline.
pub const MAX_REQUEST_BYTES: usize = 4096;
/// Maximum serialized response size, including its final newline.
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

/// Failures expose fixed messages so private seed input is never echoed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request exceeded the framing limit or was incomplete.
    #[error("invalid request frame")]
    Frame,
    /// The request was not a supported JSON object.
    #[error("invalid request JSON")]
    Json,
    /// Coordinates or sample count exceeded the supported bounds.
    #[error("invalid prediction bounds")]
    Bounds,
    /// The process could not read its input or write its output.
    #[error("prediction transport failed")]
    Io,
    /// The response could not be encoded within its size limit.
    #[error("prediction encoding failed")]
    Encoding,
}
