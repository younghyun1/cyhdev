//! Maps feature errors to the stable API error contract.

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::kcna_archive::error::{ArchiveError, UnavailableReason},
};

/// Chooses the status and error code of each feature error. A loading
/// snapshot is reported as a retryable 503 with a short `Retry-After`.
pub(super) fn map_archive_error(error: ArchiveError) -> CodeErrorResp {
    let code = match &error {
        ArchiveError::BadRequest => CodeError::INVALID_REQUEST,
        ArchiveError::NotFound => CodeError::KCNA_ARCHIVE_DOCUMENT_NOT_FOUND,
        ArchiveError::Unavailable(UnavailableReason::NotInstalled) => {
            CodeError::KCNA_ARCHIVE_NOT_INSTALLED
        }
        ArchiveError::Unavailable(UnavailableReason::Loading) => CodeError::KCNA_ARCHIVE_LOADING,
        ArchiveError::Unavailable(UnavailableReason::Failed) => CodeError::KCNA_ARCHIVE_LOAD_FAILED,
        ArchiveError::Busy => CodeError::KCNA_ARCHIVE_RELOAD_BUSY,
        ArchiveError::ReloadFailed => CodeError::KCNA_ARCHIVE_RELOAD_FAILED,
        ArchiveError::Internal => CodeError::KCNA_ARCHIVE_INTERNAL,
    };
    let response = code_err(code, &error);
    if error.is_retryable() {
        response.with_retry_after(std::time::Duration::from_secs(5))
    } else {
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn every_feature_error_has_a_stable_status_and_code() {
        let expected = [
            (ArchiveError::BadRequest, StatusCode::BAD_REQUEST, 45),
            (ArchiveError::NotFound, StatusCode::NOT_FOUND, 112),
            (
                ArchiveError::Unavailable(UnavailableReason::NotInstalled),
                StatusCode::SERVICE_UNAVAILABLE,
                113,
            ),
            (
                ArchiveError::Unavailable(UnavailableReason::Loading),
                StatusCode::SERVICE_UNAVAILABLE,
                114,
            ),
            (
                ArchiveError::Unavailable(UnavailableReason::Failed),
                StatusCode::SERVICE_UNAVAILABLE,
                115,
            ),
            (ArchiveError::Busy, StatusCode::CONFLICT, 116),
            (
                ArchiveError::ReloadFailed,
                StatusCode::INTERNAL_SERVER_ERROR,
                117,
            ),
            (
                ArchiveError::Internal,
                StatusCode::INTERNAL_SERVER_ERROR,
                118,
            ),
        ];
        for (error, status, code) in expected {
            let label = error.to_string();
            let response = map_archive_error(error);
            assert_eq!(response.http_status_code, status, "{label}");
            assert_eq!(response.error_code, code, "{label}");
            assert!(!response.success);
        }
    }
}
