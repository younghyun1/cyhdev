//! Diesel persistence for UI text synchronization and read-through.

use diesel::{
    BoolExpressionMethods, DecoratableTarget, ExpressionMethods, PgExpressionMethods, QueryDsl,
    Queryable, Selectable, SelectableHelper, upsert::excluded,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use uuid::Uuid;

use crate::{
    features::i18n::domain::{
        message::InternationalizationString,
        source_sync::{StoredUiText, UiTextSourceRow},
    },
    schema::i18n_strings,
};

/// Rows per upsert statement. Five bound values per row keep a chunk far
/// below PostgreSQL's 65,535-parameter limit while cutting 5,000-plus
/// single-row round trips to a handful.
const SOURCE_SYNC_CHUNK_ROWS: usize = 1_000;

#[derive(Queryable, Selectable)]
#[diesel(table_name = i18n_strings, check_for_backend(diesel::pg::Pg))]
struct I18nStringRecord {
    i18n_string_id: Uuid,
    i18n_string_content: String,
    i18n_string_created_at: chrono::DateTime<chrono::Utc>,
    i18n_string_created_by: Uuid,
    i18n_string_updated_at: chrono::DateTime<chrono::Utc>,
    i18n_string_updated_by: Uuid,
    i18n_string_language_code: i32,
    i18n_string_country_code: i32,
    i18n_string_country_subdivision_code: Option<String>,
    i18n_string_reference_key: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = i18n_strings, check_for_backend(diesel::pg::Pg))]
struct StoredSourceRecord {
    i18n_string_reference_key: String,
    i18n_string_country_code: i32,
    i18n_string_language_code: i32,
    i18n_string_content: String,
}

pub struct I18nRepository {
    pool: Pool<AsyncPgConnection>,
}

impl I18nRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub async fn cache_rows(&self, limit: i64) -> anyhow::Result<Vec<InternationalizationString>> {
        let mut connection = self.pool.get().await?;
        Ok(i18n_strings::table
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null())
            .order((
                i18n_strings::i18n_string_updated_at.desc(),
                i18n_strings::i18n_string_id.desc(),
            ))
            .limit(limit)
            .select(I18nStringRecord::as_select())
            .load::<I18nStringRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn bundle_rows(
        &self,
        reference_keys: &[String],
        country_code: i32,
        language_code: i32,
        fallback_country_code: i32,
        fallback_language_code: i32,
    ) -> anyhow::Result<Vec<InternationalizationString>> {
        let mut connection = self.pool.get().await?;
        Ok(i18n_strings::table
            .filter(i18n_strings::i18n_string_reference_key.eq_any(reference_keys))
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null())
            .filter(
                i18n_strings::i18n_string_country_code
                    .eq(country_code)
                    .and(i18n_strings::i18n_string_language_code.eq(language_code))
                    .or(i18n_strings::i18n_string_country_code
                        .eq(fallback_country_code)
                        .and(i18n_strings::i18n_string_language_code.eq(fallback_language_code))),
            )
            .order((
                i18n_strings::i18n_string_updated_at.desc(),
                i18n_strings::i18n_string_id.desc(),
            ))
            .select(I18nStringRecord::as_select())
            .load::<I18nStringRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Stored file-backed rows (no subdivision) for `reference_keys`.
    ///
    /// The partial unique index on key, country, and language admits one such
    /// row per key and locale, so the result is bounded by the key count times
    /// the stored locales.
    pub async fn source_texts(&self, reference_keys: &[&str]) -> anyhow::Result<Vec<StoredUiText>> {
        let mut connection = self.pool.get().await?;
        Ok(i18n_strings::table
            .filter(i18n_strings::i18n_string_reference_key.eq_any(reference_keys))
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null())
            .select(StoredSourceRecord::as_select())
            .load::<StoredSourceRecord>(&mut connection)
            .await?
            .into_iter()
            .map(|record| StoredUiText {
                reference_key: record.i18n_string_reference_key,
                country_code: record.i18n_string_country_code,
                language_code: record.i18n_string_language_code,
                content: record.i18n_string_content,
            })
            .collect())
    }

    /// Upserts `rows` and returns how many were inserted or updated.
    ///
    /// Rows are written in multi-row statements of [`SOURCE_SYNC_CHUNK_ROWS`]
    /// inside one transaction; an empty slice opens no connection. Callers pass
    /// only rows that differed when read, and the conflict update still fires
    /// only when the stored content differs, so a concurrent synchronization
    /// that wrote the same content first turns the row into a no-op and keeps
    /// its `updated_at`, which orders the cache snapshot.
    pub async fn upsert_source_rows(&self, rows: &[UiTextSourceRow<'_>]) -> anyhow::Result<usize> {
        if rows.is_empty() {
            return Ok(0);
        }
        let mut connection = self.pool.get().await?;
        let system_user_id = Uuid::nil();
        let now = chrono::Utc::now();
        connection
            .transaction::<usize, diesel::result::Error, _>(async |connection| {
                let mut written = 0;
                for chunk in rows.chunks(SOURCE_SYNC_CHUNK_ROWS) {
                    let values = chunk
                        .iter()
                        .map(|row| {
                            (
                                i18n_strings::i18n_string_content.eq(row.content),
                                i18n_strings::i18n_string_updated_by.eq(system_user_id),
                                i18n_strings::i18n_string_language_code.eq(row.language_code),
                                i18n_strings::i18n_string_country_code.eq(row.country_code),
                                i18n_strings::i18n_string_reference_key.eq(row.reference_key),
                            )
                        })
                        .collect::<Vec<_>>();
                    let upsert = diesel::insert_into(i18n_strings::table)
                        .values(values)
                        .on_conflict((
                            i18n_strings::i18n_string_reference_key,
                            i18n_strings::i18n_string_country_code,
                            i18n_strings::i18n_string_language_code,
                        ))
                        .filter_target(i18n_strings::i18n_string_country_subdivision_code.is_null())
                        .do_update()
                        .set((
                            i18n_strings::i18n_string_content
                                .eq(excluded(i18n_strings::i18n_string_content)),
                            i18n_strings::i18n_string_updated_at.eq(now),
                            i18n_strings::i18n_string_updated_by.eq(system_user_id),
                        ));
                    // `ON CONFLICT ... DO UPDATE ... WHERE`. Insert statements have
                    // no `QueryDsl`, and importing `FilterDsl` would make every
                    // table `.filter` in this file ambiguous, so it is called by path.
                    written += diesel::query_dsl::methods::FilterDsl::filter(
                        upsert,
                        i18n_strings::i18n_string_content
                            .is_distinct_from(excluded(i18n_strings::i18n_string_content)),
                    )
                    .execute(&mut *connection)
                    .await?;
                }
                Ok(written)
            })
            .await
            .map_err(Into::into)
    }
}

impl From<I18nStringRecord> for InternationalizationString {
    fn from(value: I18nStringRecord) -> Self {
        Self {
            i18n_string_id: value.i18n_string_id,
            i18n_string_content: value.i18n_string_content,
            i18n_string_created_at: value.i18n_string_created_at,
            i18n_string_created_by: value.i18n_string_created_by,
            i18n_string_updated_at: value.i18n_string_updated_at,
            i18n_string_updated_by: value.i18n_string_updated_by,
            i18n_string_language_code: value.i18n_string_language_code,
            i18n_string_country_code: value.i18n_string_country_code,
            i18n_string_country_subdivision_code: value.i18n_string_country_subdivision_code,
            i18n_string_reference_key: value.i18n_string_reference_key,
        }
    }
}
