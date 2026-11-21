//! The reader gate: superusers, and the accounts on the reader list. It
//! runs after the session check, which has already rejected accounts without
//! a verified address.

use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use super::error_map::map_archive_error;
use crate::{
    errors::code_error::{CodeError, HandlerResponse, code_err},
    features::{
        accounts::domain::role::RoleType,
        kcna_archive::{error::ArchiveError, service::archive_service::KcnaArchiveService},
    },
};

/// Whether the account may read: any superuser, and anyone on the list.
pub(super) async fn may_read(service: &KcnaArchiveService, role: RoleType, user_id: Uuid) -> bool {
    role.is_superuser() || service.admits(user_id).await
}

/// Rejects accounts that are neither superusers nor listed.
pub async fn require_archive_reader(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    request: Request<Body>,
    next: Next,
) -> HandlerResponse<Response> {
    let identity = (
        request.extensions().get::<RoleType>().copied(),
        request.extensions().get::<Uuid>().copied(),
    );
    let (role, user_id) = match identity {
        (Some(role), Some(user_id)) => (role, user_id),
        (None, _) | (_, None) => {
            return Err(code_err(
                CodeError::UNAUTHORIZED_ACCESS,
                "Missing session identity in request",
            ));
        }
    };
    if !may_read(&service, role, user_id).await {
        return Err(map_archive_error(ArchiveError::Denied));
    }
    Ok(next.run(request).await.into_response())
}
