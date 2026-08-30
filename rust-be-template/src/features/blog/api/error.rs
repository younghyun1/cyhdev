use axum::http::StatusCode;
use tracing::Level;
use crate::errors::code_error::{CodeError, CodeErrorResp, code_err};

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
        BlogError::Database(_) | BlogError::Invariant(_) => match operation {
            BlogOperation::Query => CodeError::DB_QUERY_ERROR,
            BlogOperation::Insert => CodeError::DB_INSERTION_ERROR,
            BlogOperation::Update => CodeError::DB_UPDATE_ERROR,
            BlogOperation::Delete | BlogOperation::VoteRescind => CodeError::DB_DELETION_ERROR,
        },
    };
    code_err(code, error)
}
