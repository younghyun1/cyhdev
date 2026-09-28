//! Public observations and markers; marker mutations inherit the existing administrator guards.

use super::{
    dto::MinecraftActionResult,
    map_dto::{
        MinecraftMapQuery, MinecraftWaypoint, MinecraftWaypointInput, MinecraftWaypointQuery,
    },
    map_error::map_error,
    map_response::MinecraftMapData,
};
use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::minecraft::service::world_query::WorldQueryService,
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Path, Query},
    response::IntoResponse,
    routing::{get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

pub fn public_router(state: &Arc<ServerState>) -> anyhow::Result<Router<Arc<ServerState>>> {
    let world = Arc::new(WorldQueryService::from_environment()?);
    Ok(Router::new()
        .route("/api/minecraft/map/query", post(minecraft_map_query))
        .route("/api/minecraft/map/waypoints", get(minecraft_map_waypoints))
        .layer(DefaultBodyLimit::max(4096))
        .layer(Extension(world))
        .layer(Extension(state.minecraft_waypoint_service()))
        .layer(axum::middleware::from_fn(
            crate::routers::middleware::sensitive_response::sensitive_response_headers,
        )))
}

pub fn admin_router(state: &Arc<ServerState>) -> Router<Arc<ServerState>> {
    admin_routes().layer(Extension(state.minecraft_waypoint_service()))
}

fn admin_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route(
            "/api/admin/minecraft/map/waypoints",
            post(create_minecraft_map_waypoint),
        )
        .route(
            "/api/admin/minecraft/map/waypoints/{waypoint_id}",
            patch(update_minecraft_map_waypoint).delete(delete_minecraft_map_waypoint),
        )
        .layer(DefaultBodyLimit::max(4096))
}

type Waypoints = Arc<crate::features::minecraft::service::waypoints::WaypointService>;

#[utoipa::path(post, path = "/api/minecraft/map/query", tag = "minecraft", request_body = MinecraftMapQuery, responses((status = 200, body = MinecraftMapData), (status = 400, body = CodeErrorResp), (status = 429, body = CodeErrorResp), (status = 503, body = CodeErrorResp)))]
pub async fn minecraft_map_query(
    Extension(service): Extension<Arc<WorldQueryService>>,
    Json(request): Json<MinecraftMapQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let data = service.query(request.into()).await.map_err(map_error)?;
    Ok(http_resp(MinecraftMapData::from(data), (), start))
}

#[utoipa::path(get, path = "/api/minecraft/map/waypoints", tag = "minecraft", params(MinecraftWaypointQuery), responses((status = 200, body = Vec<MinecraftWaypoint>), (status = 400, body = CodeErrorResp)))]
pub async fn minecraft_map_waypoints(
    Extension(service): Extension<Waypoints>,
    Query(query): Query<MinecraftWaypointQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let data = service.list(&query.world).await.map_err(map_error)?;
    Ok(http_resp(
        data.into_iter()
            .map(MinecraftWaypoint::from)
            .collect::<Vec<_>>(),
        (),
        start,
    ))
}

#[utoipa::path(post, path = "/api/admin/minecraft/map/waypoints", tag = "admin", request_body = MinecraftWaypointInput, responses((status = 200, body = MinecraftWaypoint), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 409, body = CodeErrorResp)))]
pub async fn create_minecraft_map_waypoint(
    Extension(actor): Extension<Uuid>,
    Extension(service): Extension<Waypoints>,
    Json(request): Json<MinecraftWaypointInput>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let data = service
        .save(actor, None, request.into())
        .await
        .map_err(map_error)?;
    Ok(http_resp(MinecraftWaypoint::from(data), (), start))
}

#[utoipa::path(patch, path = "/api/admin/minecraft/map/waypoints/{waypoint_id}", tag = "admin", params(("waypoint_id" = Uuid, Path)), request_body = MinecraftWaypointInput, responses((status = 200, body = MinecraftWaypoint), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 404, body = CodeErrorResp), (status = 409, body = CodeErrorResp)))]
pub async fn update_minecraft_map_waypoint(
    Extension(actor): Extension<Uuid>,
    Extension(service): Extension<Waypoints>,
    Path(id): Path<Uuid>,
    Json(request): Json<MinecraftWaypointInput>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let data = service
        .save(actor, Some(id), request.into())
        .await
        .map_err(map_error)?;
    Ok(http_resp(MinecraftWaypoint::from(data), (), start))
}

#[utoipa::path(delete, path = "/api/admin/minecraft/map/waypoints/{waypoint_id}", tag = "admin", params(("waypoint_id" = Uuid, Path)), responses((status = 200, body = MinecraftActionResult), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn delete_minecraft_map_waypoint(
    Extension(actor): Extension<Uuid>,
    Extension(service): Extension<Waypoints>,
    Path(id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    service.delete(actor, id).await.map_err(map_error)?;
    Ok(http_resp(
        MinecraftActionResult { acknowledged: true },
        (),
        start,
    ))
}

#[cfg(test)]
#[path = "map_tests.rs"]
mod tests;
