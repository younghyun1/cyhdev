use crate::errors::code_error::{CodeError, CodeErrorResp, code_err};
use axum::http::StatusCode;
use tracing::Level;

use super::super::error::BlogError;

#[derive(Clone, Copy)]
pub enum BlogOperation {
    Query,
    Insert,
    Update,
    Delete,
    VoteRescind,
}

const BLOG_VOTE_NOT_FOUND: CodeError = CodeError {
    success: false,
    error_code: 31,
    http_status_code: StatusCode::NOT_FOUND,
    message: "Blog vote does not exist!",
    log_level: Level::INFO,
};

pub fn map_blog_error(error: BlogError, operation: BlogOperation) -> CodeErrorResp {
    let code = match &error {
        BlogError::Pool(_) => CodeError::POOL_ERROR,
        BlogError::Unauthorized => CodeError::UNAUTHORIZED_ACCESS,
        BlogError::Forbidden => CodeError::IS_NOT_SUPERUSER,
        BlogError::DuplicateTitle => CodeError::POST_TITLE_NOT_UNIQUE,
        BlogError::PostNotFound => CodeError::POST_NOT_FOUND,
        BlogError::CommentNotFound => CodeError::COMMENT_NOT_FOUND,
        BlogError::VoteNotFound => BLOG_VOTE_NOT_FOUND,
        BlogError::InvalidInput => CodeError::INVALID_REQUEST,
        BlogError::Search(_) | BlogError::Task(_) => CodeError::DB_QUERY_ERROR,
        BlogError::WriteThrottled { .. } => CodeError::CONTENT_WRITE_THROTTLED,
        BlogError::Database(_) | BlogError::Invariant(_) => match operation {
            BlogOperation::Query => CodeError::DB_QUERY_ERROR,
            BlogOperation::Insert => CodeError::DB_INSERTION_ERROR,
            BlogOperation::Update => CodeError::DB_UPDATE_ERROR,
            BlogOperation::Delete | BlogOperation::VoteRescind => CodeError::DB_DELETION_ERROR,
        },
    };
    let retry_after = match &error {
        BlogError::WriteThrottled {
            retry_after,
            saturated,
        } => {
            tracing::warn!(
                event = "blog_write_rejected",
                capacity_saturated = *saturated,
                retry_after_seconds = retry_after.as_secs(),
                "Blog comment or vote rejected by the write budget"
            );
            Some(*retry_after)
        }
        _ => None,
    };
    let response = code_err(code, error);
    match retry_after {
        Some(retry_after) => response.with_retry_after(retry_after),
        None => response,
    }
}
