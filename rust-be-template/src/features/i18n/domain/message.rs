//! Persistence-free internationalized message value.

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct InternationalizationString {
    pub i18n_string_id: uuid::Uuid,
    pub i18n_string_content: String,
    pub i18n_string_created_at: chrono::DateTime<chrono::Utc>,
    pub i18n_string_created_by: uuid::Uuid,
    pub i18n_string_updated_at: chrono::DateTime<chrono::Utc>,
    pub i18n_string_updated_by: uuid::Uuid,
    pub i18n_string_language_code: i32,
    pub i18n_string_country_code: i32,
    pub i18n_string_country_subdivision_code: Option<String>,
    pub i18n_string_reference_key: String,
}
