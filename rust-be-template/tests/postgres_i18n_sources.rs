//! Embedded locale synchronization, database uniqueness, and cached/read-through selection.

mod support;

use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use rust_be_template::{
    features::i18n::{
        domain::{keys::REQUIRED_UI_TEXT_KEYS, locale::UiLocale, source::source_bundles},
        repository::i18n_repository::I18nRepository,
        service::i18n_service::I18nService,
    },
    schema::i18n_strings,
};
use support::database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn all_locale_sources_sync_idempotently_and_read_back_without_fallback() -> TestResult {
    run_database_test(synchronize_case).await
}

fn synchronize_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let pool = database.pool()?;
        let repository = Arc::new(I18nRepository::new(pool.clone()));
        let service = I18nService::new(Arc::clone(&repository));
        let sources = source_bundles()?;
        require(sources.len() == 8, "expected eight embedded locale sources")?;
        let expected_rows = sources.len() * REQUIRED_UI_TEXT_KEYS.len();
        require(
            service.synchronize_file_sources().await? == expected_rows,
            "source synchronization did not process every locale key",
        )?;

        let mut connection = pool.get().await?;
        let first_ids = i18n_strings::table
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null())
            .order(i18n_strings::i18n_string_id)
            .limit(i64::try_from(expected_rows + 1)?)
            .select(i18n_strings::i18n_string_id)
            .load::<Uuid>(&mut connection)
            .await?;
        drop(connection);
        require(
            first_ids.len() == expected_rows,
            "database row count differs from the embedded source count",
        )?;

        // A cold service exercises database read-through rather than only the RAM projection.
        verify_locale_readback(&service).await?;
        require(
            service.synchronize_cache().await? == expected_rows,
            "cache refresh did not load every locale key",
        )?;
        verify_locale_readback(&service).await?;

        require(
            service.synchronize_file_sources().await? == expected_rows,
            "repeated synchronization did not process every source key",
        )?;
        let mut connection = pool.get().await?;
        let second_ids = i18n_strings::table
            .filter(i18n_strings::i18n_string_country_subdivision_code.is_null())
            .order(i18n_strings::i18n_string_id)
            .limit(i64::try_from(expected_rows + 1)?)
            .select(i18n_strings::i18n_string_id)
            .load::<Uuid>(&mut connection)
            .await?;
        drop(connection);
        require(
            first_ids == second_ids,
            "source upserts duplicated or replaced stored rows",
        )?;

        for source in sources {
            let bundle = service
                .ui_text_bundle(
                    source.locale.country_code(),
                    source.locale.language_code(),
                    UiLocale::EnUs.country_code(),
                    UiLocale::EnUs.language_code(),
                    REQUIRED_UI_TEXT_KEYS,
                )
                .await?;
            require(
                bundle.len() == REQUIRED_UI_TEXT_KEYS.len(),
                "locale bundle is incomplete",
            )?;
            for entry in source.entries {
                require(
                    bundle.get(&entry.key) == Some(&entry.content),
                    "requested locale content was replaced by another locale or fallback",
                )?;
            }
        }
        Ok(())
    })
}

async fn verify_locale_readback(service: &I18nService) -> TestResult {
    for (locale, expected_save) in [
        (UiLocale::EnUs, "Save"),
        (UiLocale::KoKr, "저장"),
        (UiLocale::FrFr, "Enregistrer"),
        (UiLocale::EsEs, "Guardar"),
        (UiLocale::ZhHans, "保存"),
        (UiLocale::ZhHant, "儲存"),
        (UiLocale::JaJp, "保存"),
        (UiLocale::DeDe, "Speichern"),
    ] {
        let bundle = service
            .ui_text_bundle(
                locale.country_code(),
                locale.language_code(),
                UiLocale::EnUs.country_code(),
                UiLocale::EnUs.language_code(),
                &["common.save"],
            )
            .await?;
        if bundle.get("common.save").map(String::as_str) != Some(expected_save) {
            return Err(format!(
                "{} save label differs: expected {expected_save:?}, received {:?}",
                locale.as_tag(),
                bundle.get("common.save"),
            )
            .into());
        }
    }
    Ok(())
}
