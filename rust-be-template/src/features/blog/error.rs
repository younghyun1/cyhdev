use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::bb8::RunError;

#[derive(Debug, thiserror::Error)]
pub enum BlogError {
    #[error("database pool unavailable")]
    Pool(#[from] RunError),
    #[error("blog persistence failed")]
    Database(#[from] DieselError),
    #[error("active account authorization failed")]
    Unauthorized,
    #[error("blog post was not found")]
    PostNotFound,
    #[error("blog comment was not found")]
    CommentNotFound,
    #[error("blog vote was not found")]
    VoteNotFound,
    #[error("blog target is owned by another account")]
    Forbidden,
    #[error("blog input is invalid")]
    InvalidInput,
    #[error("blog persistence invariant failed: {0}")]
    Invariant(&'static str),
    #[error("post title conflicts with an existing slug")]
    DuplicateTitle,
    #[error("search index failed")]
    Search(#[source] anyhow::Error),
    #[error("blocking blog task failed")]
    Task(#[from] tokio::task::JoinError),
}

impl BlogError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_) => true,
            Self::Database(DieselError::DatabaseError(kind, _)) => matches!(
                *kind,
                DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
            ),
            _ => false,
        }
    }
}
