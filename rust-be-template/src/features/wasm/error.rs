//! WebAssembly feature failures independent of HTTP status selection.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::bb8::RunError;

#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("database pool unavailable")]
    Pool(#[from] RunError),
    #[error("WebAssembly persistence failed")]
    Database(#[from] DieselError),
    #[error("current account cannot mutate WebAssembly modules")]
    Unauthorized,
    #[error("WebAssembly module was not found")]
    NotFound,
    #[error("WebAssembly bundle is invalid")]
    Bundle(#[source] anyhow::Error),
    #[error("WebAssembly bundle task failed")]
    Task(#[from] tokio::task::JoinError),
    #[error("WebAssembly thumbnail processing failed")]
    Image(#[source] anyhow::Error),
    #[error("WebAssembly object-store operation failed")]
    ObjectStore(#[source] crate::util::media::object_store::ObjectStoreError),
    #[error("WebAssembly service is at its fixed work limit")]
    ServiceBusy,
}

impl WasmError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_) | Self::ServiceBusy => true,
            Self::ObjectStore(error) => error.is_retryable(),
            Self::Database(DieselError::DatabaseError(kind, _)) => matches!(
                *kind,
                DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
            ),
            _ => false,
        }
    }
}
