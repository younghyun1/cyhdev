use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use serde::{Deserialize, Deserializer, de::Error as DeError};
use uuid::Uuid;

use crate::{
    dto::responses::{blog::read_post_response::ReadPostResponse, response_data::http_resp},
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

use super::{
    error::{BlogOperation, map_blog_error},
    super::domain::post::PostLookup,
};

pub struct PostLookupKey(pub PostLookup);

impl<'de> Deserialize<'de> for PostLookupKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        if let Ok(post_id) = Uuid::parse_str(&raw) {
            return Ok(Self(PostLookup::Id(post_id)));
        }
        let slug = raw.trim().to_lowercase();
        if slug.is_empty() {
            return Err(D::Error::custom("post identifier cannot be empty"));
        }
        Ok(Self(PostLookup::Slug(slug)))
    }
}

#[utoipa::path(
    get,
    path = "/api/blog/posts/{post_id}",
    tag = "blog",
    params(("post_id" = String, Path, description = "Post UUID or slug")),
    responses(
        (status = 200, description = "Post details and comments", body = ReadPostResponse),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn read_post(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Path(lookup): Path<PostLookupKey>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let viewer_id = match auth_status {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let result = state
        .blog_service()
        .read_post(lookup.0, viewer_id)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Query))?;
    Ok(http_resp(ReadPostResponse {
        post: result.post,
        post_tags: result.post_tags,
        comments: result.comments,
        vote_state: result.vote_state,
        user_badge_info: result.user_badge_info,
    }, (), start))
}
