//! Resolves the reference rows a document points at: its translation group
//! and its category. Missing rows are created inside the batch transaction.

use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, QueryDsl, pg::Pg};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::database::{
    domain::DocumentBundle,
    models::{NewCategory, NewTranslationGroup},
    schema::{kcna_categories, kcna_translation_groups},
    types::{Language, Source, TranslationLinkMethod},
};

type QueryResult<T> = std::result::Result<T, diesel::result::Error>;
pub type CategoryKey = (Source, Language, String);

/// The category a bundle names, with the source and language that scope it.
pub fn category_key(bundle: &DocumentBundle) -> Option<CategoryKey> {
    bundle
        .category
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| (bundle.source, bundle.language, name.to_owned()))
}

/// Returns the identifier of every translation group named in the batch.
pub async fn translation_groups(
    connection: &mut AsyncPgConnection,
    changed: &[(&DocumentBundle, [u8; 32])],
) -> QueryResult<HashMap<String, Uuid>> {
    let keys = changed
        .iter()
        .filter_map(|(bundle, _)| bundle.translation_group_key.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Ok(HashMap::new());
    }
    let new_groups = keys
        .iter()
        .map(|key| NewTranslationGroup {
            translation_group_source_key: key.clone(),
            // Importers only group documents by an identifier the source
            // itself shares across language editions.
            translation_group_link_method: TranslationLinkMethod::SourceIdentifier,
        })
        .collect::<Vec<_>>();
    diesel::insert_into(kcna_translation_groups::table)
        .values(&new_groups)
        .on_conflict(kcna_translation_groups::translation_group_source_key)
        .do_nothing()
        .execute(connection)
        .await?;
    let rows = kcna_translation_groups::table
        .filter(kcna_translation_groups::translation_group_source_key.eq_any(&keys))
        .select((
            kcna_translation_groups::translation_group_source_key,
            kcna_translation_groups::translation_group_id,
        ))
        .load::<(String, Uuid)>(connection)
        .await?;
    Ok(rows.into_iter().collect())
}

/// Returns the identifier of every category named in the batch.
pub async fn categories(
    connection: &mut AsyncPgConnection,
    changed: &[(&DocumentBundle, [u8; 32])],
) -> QueryResult<HashMap<CategoryKey, Uuid>> {
    let keys = changed
        .iter()
        .filter_map(|(bundle, _)| category_key(bundle))
        .collect::<HashSet<_>>();
    if keys.is_empty() {
        return Ok(HashMap::new());
    }
    let new_categories = keys
        .iter()
        .map(|(source, language, name)| NewCategory {
            category_source: *source,
            category_language: *language,
            category_name: name.clone(),
        })
        .collect::<Vec<_>>();
    diesel::insert_into(kcna_categories::table)
        .values(&new_categories)
        .on_conflict((
            kcna_categories::category_source,
            kcna_categories::category_language,
            kcna_categories::category_name,
        ))
        .do_nothing()
        .execute(connection)
        .await?;
    // Names repeat across sources and languages, so the lookup narrows by name
    // and the exact triple is matched here.
    let names = keys
        .iter()
        .map(|(_, _, name)| name.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let rows = kcna_categories::table
        .filter(kcna_categories::category_name.eq_any(names))
        .select((
            kcna_categories::category_source,
            kcna_categories::category_language,
            kcna_categories::category_name,
            kcna_categories::category_id,
        ))
        .into_boxed::<Pg>()
        .load::<(Source, Language, String, Uuid)>(connection)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(source, language, name, id)| ((source, language, name), id))
        .filter(|(key, _)| keys.contains(key))
        .collect())
}
