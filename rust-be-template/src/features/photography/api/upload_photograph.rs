use std::sync::Arc;
use axum::{Extension, extract::{Multipart, State}};
use uuid::Uuid;
use crate::{features::photography::domain::photograph::Photograph, dto::responses::response_data::{Response as ApiResponse, http_resp},
    errors::code_error::{CodeErrorResp, HandlerResponse}, init::state::ServerState, util::time::now::tokio_now};
use super::{error::map_insertion_error, upload_request::read_upload};

#[utoipa::path(post, path = "/api/photographs/upload", tag = "photography", request_body(content_type = "multipart/form-data"),
responses((status = 200, body = Photograph), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn upload_photograph(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, mut multipart: Multipart) -> HandlerResponse<ApiResponse<Photograph, ()>> {
    let start = tokio_now(); let upload = read_upload(&mut multipart, user_id).await?;
    let photograph = state.photography_service().upload_photograph(user_id, upload).await.map_err(map_insertion_error)?;
    Ok(http_resp(photograph, (), start))
}
