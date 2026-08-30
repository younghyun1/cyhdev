use std::sync::Arc;

use axum::{extract::{Path, State}, response::IntoResponse};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::reference_data::domain::country::CountryAndSubdivisions,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/dropdown/country/{country_id}", tag = "countries",
    params(("country_id" = i32, Path, description = "ID of the country to retrieve")),
    responses(
        (status = 200, description = "Country and its subdivisions", body = CountryAndSubdivisions),
        (status = 404, description = "Country not found", body = CodeErrorResp)
    )
)]
pub async fn get_country(
    State(state): State<Arc<ServerState>>,
    Path(country_id): Path<i32>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let country = state
        .reference_data_service()
        .country(country_id)
        .await
        .ok_or_else(|| code_err(CodeError::COUNTRY_NOT_FOUND, "Country not found"))?;
    Ok(http_resp(country, (), start))
}
