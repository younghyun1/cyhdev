use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse},
    features::reference_data::domain::country::IsoCountrySubdivision,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/dropdown/country/{country_id}/subdivision", tag = "countries",
    params(("country_id" = i32, Path, description = "ID of the country to retrieve subdivisions for")),
    responses(
        (status = 200, description = "List of subdivisions for the country", body = [IsoCountrySubdivision]),
        (status = 404, description = "Country not found", body = CodeErrorResp)
    )
)]
pub async fn get_subdivisions_for_country(
    State(state): State<Arc<ServerState>>,
    Path(country_id): Path<i32>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let subdivisions = state
        .reference_data_service()
        .subdivisions(country_id)
        .await
        .ok_or(CodeError::COUNTRY_NOT_FOUND)?;
    Ok(http_resp(subdivisions, (), start))
}
