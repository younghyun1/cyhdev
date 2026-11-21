//! Round-trip checks against a real PostgreSQL 18 server.
//!
//! These run only when `KCNA_TEST_DATABASE_URL` names a disposable database,
//! because mocks cannot validate the UUIDv7 check constraints or the upsert
//! conflict targets.

use std::time::{SystemTime, UNIX_EPOCH};

use diesel::{ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;
use time::{
    Date, Duration, OffsetDateTime,
    macros::{date, offset, time},
};
use uuid::Uuid;

use crate::{
    database::{
        domain::{DocumentBody, DocumentBundle, DocumentTitle, ElementIndex, RawContent},
        migrate, pool,
        schema::{kcna_categories, kcna_documents, kcna_retrievals, kcna_translation_groups},
        store::{self, Upserted},
        test_reads::DocumentStore,
        types::{Language, PublicationPrecision, Source},
    },
    error::{Error, Result},
    importer::common::{content_sha256, sha256, validated_url},
};

struct Sample<'a> {
    name: &'a str,
    source: Source,
    language: Language,
    title: &'a str,
    body: &'a str,
}

fn bundle(prefix: &str, marker: u128, day: Date, sample: &Sample<'_>) -> Result<DocumentBundle> {
    let url = format!("{prefix}{}", sample.name);
    let raw = format!("<html>{}{}</html>", sample.title, sample.body).into_bytes();
    let invalid = |error: &dyn std::fmt::Display| Error::InvalidRecord(error.to_string());
    Ok(DocumentBundle {
        source: sample.source,
        source_record_id: Some(url.clone()),
        canonical_url: validated_url(&url)?,
        element_index: ElementIndex::try_new(0).map_err(|error| invalid(&error))?,
        language: sample.language,
        translation_group_key: Some(format!("integration:{marker}")),
        title: DocumentTitle::try_new(sample.title).map_err(|error| invalid(&error))?,
        body: Some(DocumentBody::try_new(sample.body).map_err(|error| invalid(&error))?),
        published_at: day.with_time(time!(12:00)).assume_offset(offset!(+9)),
        publication_precision: PublicationPrecision::Day,
        publication_date: day,
        published_at_raw: None,
        category: Some(format!("integration {marker}")),
        content_sha256: content_sha256(sample.title, Some(sample.body)),
        copyright_status: None,
        source_metadata: serde_json::json!({}),
        retrieved_at: OffsetDateTime::UNIX_EPOCH + Duration::days(20_000),
        requested_url: validated_url(&url)?,
        final_url: None,
        http_status: Some(200),
        media_type: Some("text/html".to_owned()),
        charset: Some("utf-8".to_owned()),
        raw_sha256: sha256(&raw),
        raw_content: RawContent::try_new(raw).map_err(|error| invalid(&error))?,
        response_headers: serde_json::json!({}),
        manifest_metadata: serde_json::json!({}),
    })
}

#[tokio::test]
async fn round_trips_documents_links_and_redating() -> Result<()> {
    let Ok(database_url) = std::env::var("KCNA_TEST_DATABASE_URL") else {
        eprintln!("KCNA_TEST_DATABASE_URL is unset; skipping PostgreSQL checks");
        return Ok(());
    };
    migrate::run(&database_url).await?;
    let pool = pool::connect(&database_url).await?;
    let documents = DocumentStore::new(pool.clone());
    // A per-run marker isolates this run's rows from anything else stored.
    let marker = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let prefix = format!("http://integration.test/{marker}/");
    let day = date!(2031 - 03 - 05);
    let korean = bundle(
        &prefix,
        marker,
        day,
        &Sample {
            name: "a",
            source: Source::KcnaKp,
            language: Language::Korean,
            title: "보도",
            body: "김정은동지께서 연설하시였다",
        },
    )?;
    let english = bundle(
        &prefix,
        marker,
        day,
        &Sample {
            name: "b",
            source: Source::KcnaKp,
            language: Language::English,
            title: "Report",
            body: "A speech was made.",
        },
    )?;
    let mirror = bundle(
        &prefix,
        marker,
        day,
        &Sample {
            name: "c",
            source: Source::KcnaCoJp,
            language: Language::Korean,
            title: "보도",
            body: "김정은동지께서 연설하시였다",
        },
    )?;
    let batch = [korean.clone(), english.clone(), mirror.clone()];
    let outcome = async {
        let mut connection = pool
            .get()
            .await
            .map_err(|error| Error::DatabasePool(error.to_string()))?;
        let stored_ids = async |connection: &mut _| {
            kcna_documents::table
                .filter(kcna_documents::document_canonical_url.like(format!("{prefix}%")))
                .select((
                    kcna_documents::document_canonical_url,
                    kcna_documents::document_id,
                    kcna_documents::document_updated_at,
                ))
                .order(kcna_documents::document_canonical_url.asc())
                .load::<(String, Uuid, OffsetDateTime)>(connection)
                .await
        };

        // The first import writes every row; an identical second import
        // writes nothing and leaves identifiers and timestamps alone.
        assert_eq!(
            store::upsert_batch(&pool, &batch).await?,
            Upserted {
                stored: 3,
                changed: 3
            }
        );
        let original = stored_ids(&mut connection).await?;
        assert_eq!(original.len(), 3);
        assert_eq!(
            store::upsert_batch(&pool, &batch).await?,
            Upserted {
                stored: 3,
                changed: 0
            }
        );
        assert_eq!(stored_ids(&mut connection).await?, original);

        // Category and translation group are resolved through their tables.
        let read = documents
            .get(original[0].1)
            .await?
            .ok_or_else(|| Error::InvalidRecord("stored document is missing".to_owned()))?;
        assert_eq!(read.title, "보도");
        assert_eq!(read.publication_date, day);
        assert_eq!(read.body.as_deref(), Some("김정은동지께서 연설하시였다"));
        assert_eq!(read.category, Some(format!("integration {marker}")));
        assert!(read.translation_group_id.is_some());
        let translations = documents.translations(&read).await?;
        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0].id, original[1].1);
        assert_eq!(translations[0].language, Language::English);

        // A copy linked to its preferred document is reported from both sides.
        diesel::update(kcna_documents::table.filter(kcna_documents::document_id.eq(original[2].1)))
            .set(kcna_documents::document_canonical_document_id.eq(Some(original[0].1)))
            .execute(&mut connection)
            .await?;
        let copies = documents.copies(&read).await?;
        assert_eq!(copies.len(), 1);
        assert_eq!(
            (copies[0].id, copies[0].source),
            (original[2].1, Source::KcnaCoJp)
        );
        let copy = documents
            .get(original[2].1)
            .await?
            .ok_or_else(|| Error::InvalidRecord("stored copy is missing".to_owned()))?;
        let back = documents.copies(&copy).await?;
        assert_eq!(
            back.iter().map(|other| other.id).collect::<Vec<_>>(),
            [original[0].1]
        );
        // The preferred copy stands in for its language in the translations.
        let from_english = documents
            .get(original[1].1)
            .await?
            .ok_or_else(|| Error::InvalidRecord("stored translation is missing".to_owned()))?;
        let korean_side = documents.translations(&from_english).await?;
        assert_eq!(
            korean_side.iter().map(|other| other.id).collect::<Vec<_>>(),
            [original[0].1]
        );

        // A changed body rewrites only that row.
        let mut edited = english.clone();
        edited.body = Some(
            DocumentBody::try_new("A longer speech was made.")
                .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        );
        edited.content_sha256 = content_sha256("Report", Some("A longer speech was made."));
        assert_eq!(
            store::upsert_batch(&pool, &[korean.clone(), edited]).await?,
            Upserted {
                stored: 2,
                changed: 1
            }
        );
        let after_edit = stored_ids(&mut connection).await?;
        assert_eq!(after_edit[0], original[0]);
        assert_eq!(after_edit[1].1, original[1].1);
        assert!(after_edit[1].2 > original[1].2);

        // A corrected date replaces the row under a new time-ordered identifier
        // instead of violating the identifier check and failing the batch.
        let corrected_day = day + Duration::days(2);
        let mut corrected = korean.clone();
        corrected.publication_date = corrected_day;
        corrected.published_at = corrected_day
            .with_time(time!(12:00))
            .assume_offset(offset!(+9));
        assert_eq!(
            store::upsert_batch(&pool, &[corrected]).await?,
            Upserted {
                stored: 1,
                changed: 1
            }
        );
        let after = stored_ids(&mut connection).await?;
        assert_eq!(after.len(), 3);
        assert_ne!(after[0].1, original[0].1);
        assert!(documents.get(original[0].1).await?.is_none());
        let moved = documents.get(after[0].1).await?;
        assert!(moved.is_some_and(|document| document.publication_date == corrected_day));
        // The copy that pointed at the replaced row stands alone again.
        let orphan = documents.get(original[2].1).await?;
        assert!(orphan.is_some_and(|document| document.canonical_document_id.is_none()));
        Ok::<(), Error>(())
    }
    .await;
    // Clean up even when an assertion path returned an error.
    let mut connection = pool
        .get()
        .await
        .map_err(|error| Error::DatabasePool(error.to_string()))?;
    diesel::delete(
        kcna_documents::table
            .filter(kcna_documents::document_canonical_url.like(format!("{prefix}%"))),
    )
    .execute(&mut connection)
    .await?;
    diesel::delete(
        kcna_retrievals::table
            .filter(kcna_retrievals::retrieval_requested_url.like(format!("{prefix}%"))),
    )
    .execute(&mut connection)
    .await?;
    diesel::delete(kcna_translation_groups::table.filter(
        kcna_translation_groups::translation_group_source_key.eq(format!("integration:{marker}")),
    ))
    .execute(&mut connection)
    .await?;
    diesel::delete(
        kcna_categories::table
            .filter(kcna_categories::category_name.eq(format!("integration {marker}"))),
    )
    .execute(&mut connection)
    .await?;
    outcome
}
