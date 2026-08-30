//! Diesel persistence for UI text synchronization and read-through.

use diesel::{
    BoolExpressionMethods, DecoratableTarget, ExpressionMethods, QueryDsl, Queryable, Selectable,
    SelectableHelper,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};
use uuid::Uuid;

use crate::{
    features::i18n::domain::{message::InternationalizationString, source::UiTextSourceBundle},
    schema::i18n_strings,
};

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

    pub async fn synchronize_sources(
        &self,
        bundles: Vec<UiTextSourceBundle>,
    ) -> anyhow::Result<usize> {
        let mut connection = self.pool.get().await?;
        let system_user_id = Uuid::nil();
        Ok(connection
            .transaction::<usize, diesel::result::Error, _>(async move |connection| {
                let mut synchronized = 0usize;
                for bundle in bundles {
                    for entry in bundle.entries {
                        let now = chrono::Utc::now();
                        diesel::insert_into(i18n_strings::table)
                            .values((
                                i18n_strings::i18n_string_content.eq(&entry.content),
                                i18n_strings::i18n_string_updated_by.eq(system_user_id),
                                i18n_strings::i18n_string_language_code
                                    .eq(bundle.locale.language_code()),
                                i18n_strings::i18n_string_country_code
                                    .eq(bundle.locale.country_code()),
                                i18n_strings::i18n_string_country_subdivision_code
                                    .eq(Option::<String>::None),
                                i18n_strings::i18n_string_reference_key.eq(&entry.key),
                            ))
                            .on_conflict((
                                i18n_strings::i18n_string_reference_key,
                                i18n_strings::i18n_string_country_code,
                                i18n_strings::i18n_string_language_code,
                            ))
                            .filter_target(
                                i18n_strings::i18n_string_country_subdivision_code.is_null(),
                            )
                            .do_update()
                            .set((
                                i18n_strings::i18n_string_content.eq(&entry.content),
                                i18n_strings::i18n_string_updated_at.eq(now),
                                i18n_strings::i18n_string_updated_by.eq(system_user_id),
                            ))
                            .execute(&mut *connection)
                            .await?;
                        synchronized = synchronized.saturating_add(1);
                    }
                }
                Ok(synchronized)
            })
            .await?)
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
