use std::sync::Arc;
use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;
use crate::{dto::responses::{photography::read_photograph_response::ReadPhotographResponse, response_data::http_resp},
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState, routers::middleware::is_logged_in::AuthStatus, util::time::now::tokio_now};
use super::error::map_photography_error;

#[utoipa::path(get, path = "/api/photographs/{photograph_id}", tag = "photography", params(("photograph_id" = Uuid, Path)),
responses((status = 200, body = ReadPhotographResponse), (status = 404, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn read_photograph(Extension(auth): Extension<AuthStatus>, State(state): State<Arc<ServerState>>, Path(photograph_id): Path<Uuid>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now(); let viewer = match auth { AuthStatus::LoggedIn(id) => Some(id), AuthStatus::LoggedOut => None };
    let detail = state.photography_service().photograph_detail(photograph_id, viewer).await.map_err(map_photography_error)?;
    Ok(http_resp(ReadPhotographResponse { photograph: detail.photograph, vote_state: detail.vote_state, comments: detail.comments, user_badge_info: detail.author_badge }, (), start))
}
