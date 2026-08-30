use crate::{errors::code_error::{CodeError, CodeErrorResp, code_err}, features::photography::error::PhotographyError};

pub(super) fn map_photography_error(error: PhotographyError) -> CodeErrorResp {
    map_with_database_code(error, CodeError::DB_QUERY_ERROR)
}

pub(super) fn map_insertion_error(error: PhotographyError) -> CodeErrorResp {
    map_with_database_code(error, CodeError::DB_INSERTION_ERROR)
}

pub(super) fn map_update_error(error: PhotographyError) -> CodeErrorResp {
    map_with_database_code(error, CodeError::DB_UPDATE_ERROR)
}

pub(super) fn map_deletion_error(error: PhotographyError) -> CodeErrorResp {
    map_with_database_code(error, CodeError::DB_DELETION_ERROR)
}

fn map_with_database_code(error: PhotographyError, database_code: CodeError) -> CodeErrorResp {
    let code = match &error {
        PhotographyError::Pool(_) => CodeError::POOL_ERROR,
        PhotographyError::Query(_) => database_code,
        PhotographyError::InactiveAccount | PhotographyError::Forbidden => CodeError::UNAUTHORIZED_ACCESS,
        PhotographyError::PhotographNotFound => CodeError::PHOTOGRAPH_NOT_FOUND,
        PhotographyError::CommentNotFound => CodeError::COMMENT_NOT_FOUND,
        PhotographyError::VoteNotFound => CodeError::UPVOTE_DOES_NOT_EXIST,
        PhotographyError::InvalidInput => CodeError::INVALID_REQUEST,
        PhotographyError::ViewCounterSaturated => database_code,
        PhotographyError::Image(_) => CodeError::COULD_NOT_PROCESS_IMAGE,
        PhotographyError::Media(_) => CodeError::FILE_UPLOAD_ERROR,
        PhotographyError::BatchEmpty => CodeError::BATCH_EMPTY,
        PhotographyError::BatchSaturated => CodeError::INVALID_REQUEST,
    };
    code_err(code, error)
}
