//! UI text cache, read-through, and source synchronization use cases.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tracing::info;

use crate::features::i18n::{
    domain::source::source_bundles,
    repository::i18n_repository::I18nRepository,
    service::cache::{I18N_CACHE_MAX_BYTES, I18N_CACHE_MAX_ENTRIES, I18nCache},
};

pub struct I18nService {
    repository: Arc<I18nRepository>,
    cache: RwLock<I18nCache>,
}

impl I18nService {
    pub fn new(repository: Arc<I18nRepository>) -> Self {
        Self {
            repository,
            cache: RwLock::new(I18nCache::new()),
        }
    }

    pub async fn synchronize_cache(&self) -> anyhow::Result<usize> {
        let start = tokio::time::Instant::now();
        let limit = i64::try_from(I18N_CACHE_MAX_ENTRIES.saturating_add(1))?;
        let mut rows = self.repository.cache_rows(limit).await?;
        let source_complete = rows.len() <= I18N_CACHE_MAX_ENTRIES;
        if !source_complete {
            rows.truncate(I18N_CACHE_MAX_ENTRIES);
        }
        let row_count = rows.len();
        let cache = I18nCache::from_rows(rows, source_complete);
        let stats = cache.stats();
        *self.cache.write().await = cache;
        info!(
            elapsed = ?start.elapsed(),
            rows_synchronized = row_count,
            entries = stats.entries,
            retained_bytes = stats.retained_bytes,
            max_entries = I18N_CACHE_MAX_ENTRIES,
            max_bytes = I18N_CACHE_MAX_BYTES,
            cache_complete = stats.complete,
            rejected_admissions = stats.rejected_admissions,
            "Synchronized UI i18n cache"
        );
        Ok(row_count)
    }

    pub async fn ui_text_bundle(
        &self,
        country_code: i32,
        language_code: i32,
        fallback_country_code: i32,
        fallback_language_code: i32,
        required_keys: &[&str],
    ) -> anyhow::Result<HashMap<String, String>> {
        let (cached, cache_complete) = {
            let cache = self.cache.read().await;
            (
                cache.ui_text_bundle(
                    country_code,
                    language_code,
                    fallback_country_code,
                    fallback_language_code,
                    required_keys,
                ),
                cache.is_complete(),
            )
        };
        if cache_complete && cached.len() == required_keys.len() {
            return Ok(cached);
        }
        let query_keys = if cache_complete {
            required_keys
                .iter()
                .filter(|key| !cached.contains_key(**key))
                .map(|key| (*key).to_owned())
                .collect::<Vec<_>>()
        } else {
            required_keys.iter().map(|key| (*key).to_owned()).collect()
        };
        if query_keys.is_empty() {
            return Ok(cached);
        }
        self.cache.read().await.record_database_read_through();
        let rows = self
            .repository
            .bundle_rows(
                &query_keys,
                country_code,
                language_code,
                fallback_country_code,
                fallback_language_code,
            )
            .await?;
        let query_key_refs = query_keys.iter().map(String::as_str).collect::<Vec<_>>();
        let database_texts = I18nCache::ui_text_bundle_from_rows(
            &rows,
            country_code,
            language_code,
            fallback_country_code,
            fallback_language_code,
            &query_key_refs,
        );
        self.cache.write().await.admit_rows(&rows);
        if cache_complete {
            let mut result = cached;
            result.extend(database_texts);
            Ok(result)
        } else {
            Ok(database_texts)
        }
    }

    pub async fn synchronize_file_sources(&self) -> anyhow::Result<usize> {
        let start = tokio::time::Instant::now();
        let synchronized = self
            .repository
            .synchronize_sources(source_bundles()?)
            .await?;
        info!(
            elapsed = ?start.elapsed(),
            rows_synchronized = synchronized,
            "Synchronized file-backed UI i18n source data"
        );
        Ok(synchronized)
    }
}
