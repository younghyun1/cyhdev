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
    #[error("live-chat moderation requires current superuser authority")]
    Forbidden,
    #[error("live-chat cursor was not found")]
    InvalidCursor,
    #[error("operating-system entropy unavailable for the live-chat guest identity key")]
    Entropy(#[source] getrandom::Error),
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
            Self::Unauthorized | Self::Forbidden | Self::InvalidCursor | Self::Entropy(_) => false,
        }
    }
}
