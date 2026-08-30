use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::reference_data::domain::language::IsoLanguage,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/dropdown/language/{language_id}", tag = "countries",
    params(("language_id" = i32, Path, description = "ID of the language to retrieve")),
    responses(
        (status = 200, description = "Language information", body = IsoLanguage),
        (status = 404, description = "Language not found", body = CodeErrorResp)
    )
)]
pub async fn get_language(
    State(state): State<Arc<ServerState>>,
    Path(language_id): Path<i32>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let language = state
        .reference_data_service()
        .language(language_id)
        .await
        .ok_or_else(|| code_err(CodeError::LANGUAGE_NOT_FOUND, "Language not found"))?;
    Ok(http_resp(language, (), start))
}
