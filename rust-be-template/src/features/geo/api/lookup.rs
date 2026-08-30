use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::geo::domain::geo_ip::IpInfo,
    init::state::ServerState,
    util::{extract::client_ip::extract_client_ip, time::now::tokio_now},
};

#[utoipa::path(get, path = "/api/geo-ip-info/{ip_address}", tag = "geo",
    params(("ip_address" = String, Path, description = "IP address to lookup")),
    responses(
        (status = 200, description = "IP geo-information", body = IpInfo),
        (status = 400, description = "Invalid IP address", body = CodeErrorResp)
    )
)]
pub async fn lookup_ip_info(
    State(state): State<Arc<ServerState>>,
    Path(ip_address): Path<String>,
) -> HandlerResponse<impl IntoResponse> {
    lookup_path(state, ip_address).await
}

#[utoipa::path(get, path = "/api/geolocate/{ip_address}", tag = "server",
    params(("ip_address" = String, Path, description = "IP address to lookup")),
    responses(
        (status = 200, description = "IP location information", body = IpInfo),
        (status = 400, description = "Invalid IP address", body = CodeErrorResp)
    )
)]
pub async fn lookup_ip_location(
    State(state): State<Arc<ServerState>>,
    Path(ip_address): Path<String>,
) -> HandlerResponse<impl IntoResponse> {
    lookup_path(state, ip_address).await
}

async fn lookup_path(
    state: Arc<ServerState>,
    raw_ip: String,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let ip = raw_ip
        .parse::<IpAddr>()
        .map_err(|error| code_err(CodeError::INVALID_IP_ADDRESS, error))?;
    let info = state
        .geo_service()
        .lookup(ip)
        .ok_or_else(|| code_err(CodeError::INVALID_IP_ADDRESS, "IP geo info not found"))?;
    Ok(http_resp(info, (), start))
}

#[utoipa::path(get, path = "/api/geo-ip-info/me", tag = "geo", responses(
    (status = 200, description = "Client's IP geo-information", body = IpInfo),
    (status = 400, description = "Could not determine client IP", body = CodeErrorResp)
))]
pub async fn lookup_my_ip_info(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let client_ip = extract_client_ip(&headers, socket).unwrap_or_else(|| socket.ip());
    let info = state
        .geo_service()
        .lookup(client_ip)
        .unwrap_or_else(|| IpInfo {
            ip: client_ip.to_string(),
            country_code: "XX".to_owned(),
            country_name: "Unknown".to_owned(),
            state: String::new(),
            city: String::new(),
            postal: String::new(),
            latitude: 0.0,
            longitude: 0.0,
        });
    Ok(http_resp(info, (), start))
}
