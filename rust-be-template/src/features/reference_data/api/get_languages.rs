use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::reference_data::domain::language::IsoLanguage,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/dropdown/language", tag = "countries", responses(
    (status = 200, description = "List of languages", body = [IsoLanguage]),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn get_languages(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    Ok(http_resp(
        state.reference_data_service().languages().await,
        (),
        start,
    ))
}
