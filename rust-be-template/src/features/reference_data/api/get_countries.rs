use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use utoipa::ToSchema;

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::reference_data::domain::country::IsoCountry,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[derive(ToSchema)]
pub struct GetCountriesResponse {
    pub countries: Vec<IsoCountry>,
}

#[utoipa::path(get, path = "/api/dropdown/country", tag = "countries", responses(
    (status = 200, description = "List of countries", body = GetCountriesResponse),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn get_countries(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    Ok(http_resp(
        state.reference_data_service().countries_json().await,
        (),
        start,
    ))
}
