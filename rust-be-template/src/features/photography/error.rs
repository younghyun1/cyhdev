use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::bb8::RunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhotographyError {
    #[error("photography database pool unavailable")]
    Pool(#[source] RunError),
    #[error("photography database query failed")]
    Query(#[from] DieselError),
    #[error("authenticated account is inactive")]
    InactiveAccount,
    #[error("photography operation is not authorized")]
    Forbidden,
    #[error("photograph was not found")]
    PhotographNotFound,
    #[error("photograph comment was not found")]
    CommentNotFound,
    #[error("photograph vote was not found")]
    VoteNotFound,
    #[error("photography input is invalid")]
    InvalidInput,
    #[error("photograph view counter is saturated")]
    ViewCounterSaturated,
    #[error("photograph image processing failed")]
    Image(#[source] anyhow::Error),
    #[error("photograph object persistence failed")]
    Media(#[source] anyhow::Error),
    #[error("photograph batch contains no files")]
    BatchEmpty,
    #[error("photograph batch registry is saturated")]
    BatchSaturated,
}

impl PhotographyError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_) => true,
            Self::Query(diesel::result::Error::DatabaseError(kind, _)) => matches!(
                *kind,
                diesel::result::DatabaseErrorKind::SerializationFailure
                    | diesel::result::DatabaseErrorKind::ClosedConnection
            ),
            _ => false,
        }
    }
}
