use super::error::map_photography_error;
use crate::{
    dto::responses::{
        photography::get_photograph_response::{
            GetPhotographsResponse, PaginationMeta, PhotographItem,
        },
        response_data::http_resp,
    },
    errors::code_error::HandlerResponse,
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use std::{collections::HashMap, sync::Arc};

#[utoipa::path(get, path = "/api/photographs/get", tag = "photography", params(("page" = Option<i64>, Query), ("page_size" = Option<i64>, Query)), responses((status = 200, body = GetPhotographsResponse), (status = 500)))]
pub async fn get_photographs(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = params
        .get("page")
        .and_then(|value| value.parse().ok())
        .filter(|value: &i64| *value > 0)
        .unwrap_or(1);
    let page_size = params
        .get("page_size")
        .and_then(|value| value.parse().ok())
        .filter(|value: &i64| (1..=100).contains(value))
        .unwrap_or(20);
    let result = state
        .photography_service()
        .photographs(page, page_size)
        .await
        .map_err(map_photography_error)?;
    let total_pages = if result.total_items == 0 {
        0
    } else {
        ((result.total_items + page_size - 1) / page_size).max(1)
    };
    let items = result
        .items
        .into_iter()
        .map(|item| PhotographItem {
            photograph_id: item.photograph_id,
            user_id: item.user_id,
            photograph_shot_at: item.photograph_shot_at,
            photograph_created_at: item.photograph_created_at,
            photograph_updated_at: item.photograph_updated_at,
            photograph_image_type: item.photograph_image_type,
            photograph_is_on_cloud: item.photograph_is_on_cloud,
            photograph_link: item.photograph_link,
            photograph_comments: item.photograph_comments,
            photograph_lat: item.photograph_lat,
            photograph_lon: item.photograph_lon,
            photograph_thumbnail_link: item.photograph_thumbnail_link,
            photograph_view_count: item.photograph_view_count,
            photograph_total_upvotes: item.photograph_total_upvotes,
            photograph_total_downvotes: item.photograph_total_downvotes,
        })
        .collect();
    Ok(http_resp(
        GetPhotographsResponse {
            items,
            pagination: PaginationMeta {
                page,
                page_size,
                total_items: result.total_items,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1 && total_pages > 0,
            },
        },
        (),
        start,
    ))
}
