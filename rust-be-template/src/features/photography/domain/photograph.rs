//! Persistence-independent photograph values.

use super::social::PhotographComment;
use super::social::PhotographCommentResponse;
use crate::features::blog::domain::post::UserBadgeInfo;
use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    features::blog::domain::vote::VoteState,
};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum PhotographContext {
    Photography,
    Post,
}

impl PhotographContext {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "0" | "photography" | "portfolio" | "gallery" => Some(Self::Photography),
            "1" | "post" | "posts" | "blog" | "editor" => Some(Self::Post),
            _ => None,
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        Self::parse(value)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Photograph {
    pub photograph_id: Uuid,
    pub user_id: Uuid,
    pub photograph_shot_at: Option<DateTime<Utc>>,
    pub photograph_created_at: DateTime<Utc>,
    pub photograph_updated_at: DateTime<Utc>,
    pub photograph_image_type: i32,
    pub photograph_is_on_cloud: bool,
    pub photograph_link: String,
    pub photograph_comments: String,
    pub photograph_lat: f64,
    pub photograph_lon: f64,
    pub photograph_thumbnail_link: String,
    pub photograph_context: PhotographContext,
    pub photograph_view_count: i64,
    pub photograph_total_upvotes: i64,
    pub photograph_total_downvotes: i64,
}

impl Photograph {
    pub fn anonymize_deleted_owner(&mut self) {
        self.user_id = Uuid::nil();
        self.photograph_lat = 0.0;
        self.photograph_lon = 0.0;
    }
}

pub struct NewPhotograph {
    pub user_id: Uuid,
    pub photograph_shot_at: Option<DateTime<Utc>>,
    pub photograph_image_type: i32,
    pub photograph_context: PhotographContext,
    pub photograph_is_on_cloud: bool,
    pub photograph_link: String,
    pub photograph_comments: String,
    pub photograph_lat: f64,
    pub photograph_lon: f64,
    pub photograph_thumbnail_link: String,
}

pub struct PhotographPage {
    pub items: Vec<Photograph>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
}

pub struct PhotographDetail {
    pub photograph: Photograph,
    pub comments: Vec<(PhotographComment, VoteState)>,
    pub vote_state: VoteState,
    pub authors: HashMap<Uuid, PublicAuthor>,
    pub owner_user_id: Uuid,
}

pub struct PresentedPhotographDetail {
    pub photograph: Photograph,
    pub comments: Vec<PhotographCommentResponse>,
    pub vote_state: VoteState,
    pub author_badge: UserBadgeInfo,
}
