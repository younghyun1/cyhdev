use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::bb8::RunError;

#[derive(Debug, thiserror::Error)]
pub enum LiveChatError {
    #[error("database pool unavailable")]
    Pool(#[from] RunError),
    #[error("live-chat persistence failed")]
    Database(#[from] DieselError),
    #[error("active account authorization failed")]
    Unauthorized,
    #[error("live-chat cursor was not found")]
    InvalidCursor,
}

impl LiveChatError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_) => true,
            Self::Database(DieselError::DatabaseError(kind, _)) => matches!(
                *kind,
                DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
            ),
            Self::Database(_) => false,
            Self::Unauthorized | Self::InvalidCursor => false,
        }
    }
}
