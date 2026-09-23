//! Diesel persistence for UI text synchronization and read-through.

use diesel::{
    BoolExpressionMethods, DecoratableTarget, ExpressionMethods, PgExpressionMethods, QueryDsl,
    Queryable, Selectable, SelectableHelper, upsert::excluded,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use uuid::Uuid;

use crate::{
    features::i18n::domain::{message::InternationalizationString, source::UiTextSourceBundle},
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

    /// Upserts every embedded source entry and returns how many were processed.
    ///
    /// Rows are written in multi-row statements of [`SOURCE_SYNC_CHUNK_ROWS`]
    /// inside one transaction. The conflict update only fires when the stored
    /// content differs, so an unchanged catalog rewrites no rows and keeps
    /// each row's `updated_at`, which orders the cache snapshot.
    pub async fn synchronize_sources(
        &self,
        bundles: Vec<UiTextSourceBundle>,
    ) -> anyhow::Result<usize> {
        let rows = bundles
            .iter()
            .flat_map(|bundle| {
                bundle.entries.iter().map(|entry| {
                    (
                        entry.content.as_str(),
                        bundle.locale.language_code(),
                        bundle.locale.country_code(),
                        entry.key.as_str(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut connection = self.pool.get().await?;
        let system_user_id = Uuid::nil();
        let now = chrono::Utc::now();
        connection
            .transaction::<(), diesel::result::Error, _>(async |connection| {
                for chunk in rows.chunks(SOURCE_SYNC_CHUNK_ROWS) {
                    let values = chunk
                        .iter()
                        .map(|(content, language, country, key)| {
                            (
                                i18n_strings::i18n_string_content.eq(*content),
                                i18n_strings::i18n_string_updated_by.eq(system_user_id),
                                i18n_strings::i18n_string_language_code.eq(*language),
                                i18n_strings::i18n_string_country_code.eq(*country),
                                i18n_strings::i18n_string_reference_key.eq(*key),
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
                    diesel::query_dsl::methods::FilterDsl::filter(
                        upsert,
                        i18n_strings::i18n_string_content
                            .is_distinct_from(excluded(i18n_strings::i18n_string_content)),
                    )
                    .execute(&mut *connection)
                    .await?;
                }
                Ok(())
            })
            .await?;
        Ok(rows.len())
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
