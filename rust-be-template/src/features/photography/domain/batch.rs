//! Persistence-free photograph batch values.

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ProcessingStatus {
    Queued, Encoding, Uploading, Persisting,
    Completed { photograph_id: Uuid, photograph_link: String, thumbnail_link: String },
    Failed { reason: String },
}

impl ProcessingStatus { pub fn is_terminal(&self) -> bool { matches!(self, Self::Completed { .. } | Self::Failed { .. }) } }

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub item_id: Uuid,
    pub original_file_name: Option<String>,
    pub original_size_bytes: u64,
    pub status: ProcessingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct BatchPipelineItem {
    pub item_id: Uuid,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub comments: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub struct BatchAcceptedItem { pub item_id: Uuid, pub file_name: Option<String> }
pub struct BatchAccepted { pub batch_id: Uuid, pub total: usize, pub items: Vec<BatchAcceptedItem> }
