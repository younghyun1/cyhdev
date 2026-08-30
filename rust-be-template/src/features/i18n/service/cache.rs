//! Bounded process-local UI text cache owned by the i18n service.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::features::i18n::domain::message::InternationalizationString;

pub const I18N_CACHE_MAX_ENTRIES: usize = 50_000;
pub const I18N_CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;
const I18N_CACHE_ENTRY_OVERHEAD_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct I18nCacheKey {
    reference_key: String,
    country_code: i32,
    language_code: i32,
}

impl I18nCacheKey {
    fn new(reference_key: &str, country_code: i32, language_code: i32) -> Self {
        Self { reference_key: reference_key.to_owned(), country_code, language_code }
    }
}

#[derive(Clone, Copy)]
pub struct I18nCacheStats {
    pub entries: usize,
    pub retained_bytes: usize,
    pub complete: bool,
    pub hits: u64,
    pub misses: u64,
    pub rejected_admissions: u64,
    pub database_read_throughs: u64,
}

pub struct I18nCache {
    entries: HashMap<I18nCacheKey, String>,
    retained_bytes: usize,
    complete: bool,
    hits: AtomicU64,
    misses: AtomicU64,
    rejected_admissions: AtomicU64,
    database_read_throughs: AtomicU64,
}

impl Default for I18nCache {
    fn default() -> Self {
        Self::new()
    }
}

impl I18nCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            retained_bytes: 0,
            complete: true,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            rejected_admissions: AtomicU64::new(0),
            database_read_throughs: AtomicU64::new(0),
        }
    }

    pub fn from_rows(rows: Vec<InternationalizationString>, source_complete: bool) -> Self {
        let mut cache = Self::new();
        cache.complete = source_complete;
        cache.admit_rows(&rows);
        cache
    }

    pub fn admit_rows(&mut self, rows: &[InternationalizationString]) {
        for row in rows {
            self.admit_row(row);
        }
    }

    fn admit_row(&mut self, row: &InternationalizationString) {
        if row.i18n_string_country_subdivision_code.is_some() {
            return;
        }
        let key = I18nCacheKey::new(
            &row.i18n_string_reference_key,
            row.i18n_string_country_code,
            row.i18n_string_language_code,
        );
        let estimated_bytes = key.reference_key.len()
            .saturating_add(row.i18n_string_content.len())
            .saturating_add(I18N_CACHE_ENTRY_OVERHEAD_BYTES);
        if let Some(previous) = self.entries.remove(&key) {
            self.retained_bytes = self.retained_bytes.saturating_sub(
                key.reference_key
                    .len()
                    .saturating_add(previous.len())
                    .saturating_add(I18N_CACHE_ENTRY_OVERHEAD_BYTES),
            );
        }
        if self.entries.len() >= I18N_CACHE_MAX_ENTRIES
            || estimated_bytes > I18N_CACHE_MAX_BYTES.saturating_sub(self.retained_bytes)
        {
            self.complete = false;
            self.rejected_admissions.fetch_add(1, Ordering::Relaxed);
            return;
        }
        self.retained_bytes = self.retained_bytes.saturating_add(estimated_bytes);
        self.entries.insert(key, row.i18n_string_content.clone());
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn record_database_read_through(&self) {
        self.database_read_throughs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn ui_text_bundle(
        &self,
        country_code: i32,
        language_code: i32,
        fallback_country_code: i32,
        fallback_language_code: i32,
        required_keys: &[&str],
    ) -> HashMap<String, String> {
        let mut texts = HashMap::with_capacity(required_keys.len());
        for reference_key in required_keys {
            let primary = I18nCacheKey::new(reference_key, country_code, language_code);
            let fallback = I18nCacheKey::new(
                reference_key,
                fallback_country_code,
                fallback_language_code,
            );
            match self.entries.get(&primary).or_else(|| self.entries.get(&fallback)) {
                Some(text) => {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    texts.insert((*reference_key).to_owned(), text.clone());
                }
                None => {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        texts
    }

    pub fn ui_text_bundle_from_rows(
        rows: &[InternationalizationString],
        country_code: i32,
        language_code: i32,
        fallback_country_code: i32,
        fallback_language_code: i32,
        required_keys: &[&str],
    ) -> HashMap<String, String> {
        let mut entries = HashMap::<I18nCacheKey, String>::with_capacity(rows.len());
        for row in rows {
            if row.i18n_string_country_subdivision_code.is_some() {
                continue;
            }
            entries
                .entry(I18nCacheKey::new(
                    &row.i18n_string_reference_key,
                    row.i18n_string_country_code,
                    row.i18n_string_language_code,
                ))
                .or_insert_with(|| row.i18n_string_content.clone());
        }
        let mut texts = HashMap::with_capacity(required_keys.len());
        for reference_key in required_keys {
            let primary = I18nCacheKey::new(reference_key, country_code, language_code);
            let fallback = I18nCacheKey::new(
                reference_key,
                fallback_country_code,
                fallback_language_code,
            );
            if let Some(text) = entries.get(&primary).or_else(|| entries.get(&fallback)) {
                texts.insert((*reference_key).to_owned(), text.clone());
            }
        }
        texts
    }

    pub fn stats(&self) -> I18nCacheStats {
        I18nCacheStats {
            entries: self.entries.len(),
            retained_bytes: self.retained_bytes,
            complete: self.complete,
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            rejected_admissions: self.rejected_admissions.load(Ordering::Relaxed),
            database_read_throughs: self.database_read_throughs.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::I18nCache;
    use crate::features::i18n::domain::message::InternationalizationString;

    fn row(country: i32, language: i32, content: &str) -> InternationalizationString {
        let now = Utc::now();
        InternationalizationString {
            i18n_string_id: Uuid::now_v7(),
            i18n_string_content: content.to_owned(),
            i18n_string_created_at: now,
            i18n_string_created_by: Uuid::nil(),
            i18n_string_updated_at: now,
            i18n_string_updated_by: Uuid::nil(),
            i18n_string_language_code: language,
            i18n_string_country_code: country,
            i18n_string_country_subdivision_code: None,
            i18n_string_reference_key: "common.save".to_owned(),
        }
    }

    #[test]
    fn requested_locale_wins_and_fallback_fills_a_miss() {
        let cache = I18nCache::from_rows(
            vec![row(840, 41, "Save"), row(410, 86, "저장")],
            true,
        );
        let korean = cache.ui_text_bundle(410, 86, 840, 41, &["common.save"]);
        let fallback = cache.ui_text_bundle(124, 41, 840, 41, &["common.save"]);
        assert_eq!(korean.get("common.save").map(String::as_str), Some("저장"));
        assert_eq!(fallback.get("common.save").map(String::as_str), Some("Save"));
    }
}
