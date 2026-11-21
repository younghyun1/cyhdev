//! Status and reload handlers.

use std::sync::Arc;

use axum::{Extension, response::IntoResponse};

use super::{error_map::map_archive_error, status_dto::ArchiveStatusResponse};
use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::kcna_archive::service::archive_service::KcnaArchiveService,
    util::time::now::tokio_now,
};

/// Reports the service state and, when a snapshot is loaded, its summary.
///
/// A snapshot that is not installed, still loading, or failed to load is not
/// a transport error here: the answer is 200 with `state` set, so the page can
/// show the condition. The data endpoints answer 503 with a distinct error code
/// in those states.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/status",
    tag = "kcna_archive",
    responses(
        (status = 200, description = "Service state and snapshot summary", body = ArchiveStatusResponse),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 500, body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_status(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let status = service.status().await.map_err(map_archive_error)?;
    Ok(http_resp(ArchiveStatusResponse::from(status), (), start))
}

/// Reads the snapshot from disk again and swaps it in once it is ready.
///
/// State-changing, so it sits behind the trusted-origin check like the other
/// administrative writes. The previous snapshot keeps serving while the load
/// runs and after a failure. Answers 409 when a load is already running and
/// 500 when the reload fails. Replies with the status after the swap.
#[utoipa::path(
    post,
    path = "/api/admin/kcna-archive/reload",
    tag = "kcna_archive",
    responses(
        (status = 200, description = "Reloaded; the status of the new snapshot", body = ArchiveStatusResponse),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 409, description = "A load is already running", body = CodeErrorResp),
        (status = 500, description = "The reload failed; the previous snapshot keeps serving", body = CodeErrorResp),
        (status = 503, description = "No snapshot is installed", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_reload(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    service.reload().await.map_err(map_archive_error)?;
    let status = service.status().await.map_err(map_archive_error)?;
    Ok(http_resp(ArchiveStatusResponse::from(status), (), start))
}
