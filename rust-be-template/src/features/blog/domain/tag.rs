use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Serialize, ToSchema)]
pub struct Tag {
    pub tag_id: i16,
    pub tag_name: String,
}
