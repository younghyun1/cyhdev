//! Access handlers: the caller's own access, and the reader list.

use std::sync::Arc;

use axum::{Extension, Json, extract::Path, response::IntoResponse};
use uuid::Uuid;

use super::{
    access_dto::{ArchiveAccessResponse, ArchiveReaderGrant, ArchiveReadersResponse},
    error_map::map_archive_error,
    guard::may_read,
};
use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::{
        accounts::domain::role::RoleType,
        kcna_archive::service::archive_service::KcnaArchiveService,
    },
    util::time::now::tokio_now,
};

/// Tells a signed-in account whether the archive is open to it.
///
/// The site uses the answer to decide whether to show the archive at all, so
/// it is 200 for every verified account, including those it turns away.
#[utoipa::path(
    get,
    path = "/api/kcna-archive/access",
    tag = "kcna_archive",
    responses(
        (status = 200, description = "Whether the caller may read the archive", body = ArchiveAccessResponse),
        (status = 400, description = "The account's address is not verified", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_access(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Extension(role): Extension<RoleType>,
    Extension(user_id): Extension<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let answer = ArchiveAccessResponse {
        can_read: may_read(&service, role, user_id).await,
        admin_only: role.is_superuser() && !service.has_readers().await,
    };
    Ok(http_resp(answer, (), start))
}

/// Lists the accounts allowed to read the archive.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/readers",
    tag = "kcna_archive",
    responses(
        (status = 200, description = "The reader list", body = ArchiveReadersResponse),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 500, description = "The list could not be read", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_readers(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let readers = service.readers().await.map_err(map_archive_error)?;
    Ok(http_resp(ArchiveReadersResponse::from(readers), (), start))
}

/// Adds an account to the reader list.
///
/// State-changing, so it sits behind the trusted-origin check. Adding an
/// account that is already listed changes nothing and answers 200. The change
/// applies to the account's next request.
#[utoipa::path(
    post,
    path = "/api/admin/kcna-archive/readers",
    tag = "kcna_archive",
    request_body = ArchiveReaderGrant,
    responses(
        (status = 200, description = "The reader list after the change", body = ArchiveReadersResponse),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 404, description = "No such active account", body = CodeErrorResp),
        (status = 409, description = "The list is full", body = CodeErrorResp),
        (status = 500, description = "The list could not be changed", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_grant_reader(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Extension(actor): Extension<Uuid>,
    Json(grant): Json<ArchiveReaderGrant>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let readers = service
        .grant_reader(grant.user_id, actor)
        .await
        .map_err(map_archive_error)?;
    Ok(http_resp(ArchiveReadersResponse::from(readers), (), start))
}

/// Removes an account from the reader list.
///
/// State-changing, so it sits behind the trusted-origin check. Removing an
/// account that is not listed changes nothing and answers 200.
#[utoipa::path(
    delete,
    path = "/api/admin/kcna-archive/readers/{user_id}",
    tag = "kcna_archive",
    params(("user_id" = Uuid, Path, description = "The account to remove")),
    responses(
        (status = 200, description = "The reader list after the change", body = ArchiveReadersResponse),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 500, description = "The list could not be changed", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_revoke_reader(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Extension(actor): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let readers = service
        .revoke_reader(user_id, actor)
        .await
        .map_err(map_archive_error)?;
    Ok(http_resp(ArchiveReadersResponse::from(readers), (), start))
}
