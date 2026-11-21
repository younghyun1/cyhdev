//! The synthetic snapshot is checked against the real read-only service, so the
//! numbers the campaign scenario asserts cannot drift from the corpus.

use std::{collections::BTreeSet, os::unix::fs::symlink};

use kcna_archive_core::service::{archive::Archive, export::ExportFormat, params::QueryParams};
use serde_json::Value;

use super::{build, document_id, parameters, write};
use crate::kcna_corpus::{ENTRIES, ID_BASE};

fn archive() -> anyhow::Result<(tempfile::TempDir, Archive)> {
    let directory = tempfile::tempdir()?;
    build(directory.path())?;
    let archive = Archive::load_into_memory(directory.path())?;
    Ok((directory, archive))
}

fn query(text: &str) -> QueryParams {
    QueryParams {
        q: text.to_owned(),
        sort: Some("oldest".to_owned()),
        ..QueryParams::default()
    }
}

/// Counts are written as strings, the only parameter type the campaign admits.
fn number(parameters: &Value, name: &str) -> anyhow::Result<u64> {
    let Some(text) = parameters[name].as_str() else {
        anyhow::bail!("parameter {name} is missing or not a string");
    };
    Ok(text.parse::<u64>()?)
}

/// The campaign driver rejects a campaign whose parameters are not all strings.
#[test]
fn every_parameter_is_a_string() {
    assert!(
        parameters()
            .as_object()
            .is_some_and(|object| !object.is_empty() && object.values().all(Value::is_string))
    );
}

#[test]
fn corpus_is_ordered_unique_and_spans_several_years() {
    let numbers: BTreeSet<u128> = ENTRIES.iter().map(|entry| entry.number).collect();
    assert_eq!(numbers.len(), ENTRIES.len());
    assert!(ENTRIES.windows(2).all(|pair| {
        let (a, b) = (&pair[0], &pair[1]);
        (a.date.0, a.date.1 as u8, a.date.2) <= (b.date.0, b.date.1 as u8, b.date.2)
    }));
    let years: BTreeSet<i32> = ENTRIES.iter().map(|entry| entry.date.0).collect();
    assert!(years.len() >= 8);
    assert!(ENTRIES.len() <= 36);
    assert!(
        ENTRIES
            .iter()
            .all(|entry| entry.title.contains("합성") || entry.title.starts_with("Synthetic"))
    );
}

#[test]
fn status_search_and_document_match_the_scenario_parameters() -> anyhow::Result<()> {
    let (_directory, archive) = archive()?;
    let parameters = parameters();
    let status = archive.status()?;
    assert_eq!(status.indexed, number(&parameters, "kcna_indexed")?);
    assert_eq!(
        (status.documents, status.copies, status.translation_groups),
        (25, 1, 1)
    );

    let found = archive.search(&query("lantern"))?;
    assert_eq!(
        found.total as u64,
        number(&parameters, "kcna_lantern_total")?
    );
    assert_eq!(found.total, 5);
    let first = found
        .items
        .first()
        .ok_or_else(|| anyhow::anyhow!("no hit"))?;
    assert_eq!(first.id, document_id().to_string());
    assert!(first.title.iter().any(|segment| segment.hit));
    let korean = archive.search(&query("등불"))?;
    assert_eq!(
        korean.total as u64,
        number(&parameters, "kcna_korean_lantern_total")?
    );
    assert_eq!(archive.search(&query("Aldermoor"))?.total, 1);

    let document = archive.document(&first.id, "lantern")?;
    assert_eq!(
        document.id,
        parameters["documentId"].as_str().unwrap_or_default()
    );
    assert_eq!(document.translations.len(), 1);
    assert_eq!(
        document.translations[0].id,
        parameters["kcna_translation_id"]
            .as_str()
            .unwrap_or_default()
    );
    assert_eq!(document.translations[0].language.code(), "kor");
    assert_eq!(document.copies.len(), 1);
    assert_eq!(document.copies[0].source.code(), "kcna_co_jp");
    // The Korean edition links back to the English one.
    let back = archive.document(&document.translations[0].id, "")?;
    assert_eq!(
        back.translations.first().map(|link| link.id.as_str()),
        Some(first.id.as_str())
    );
    Ok(())
}

#[test]
fn histogram_trends_and_export_have_several_buckets() -> anyhow::Result<()> {
    let (_directory, archive) = archive()?;
    let parameters = parameters();
    let years = number(&parameters, "kcna_years")? as usize;
    let yearly = QueryParams {
        bucket: Some("year".to_owned()),
        ..QueryParams::default()
    };
    let histogram = archive.histogram(&yearly)?;
    assert_eq!(histogram.buckets.len(), years);
    assert_eq!(histogram.buckets[0].start, "2019-01-01");
    assert_eq!(
        histogram
            .buckets
            .iter()
            .map(|bucket| bucket.count)
            .sum::<u64>(),
        ENTRIES.len() as u64
    );

    let trends = archive.trends(&yearly, vec!["lantern".to_owned(), "harbor".to_owned()])?;
    assert_eq!(trends.starts.len(), years);
    assert_eq!(trends.totals.iter().sum::<u64>(), ENTRIES.len() as u64);
    assert_eq!(trends.series.len(), 2);
    assert_eq!(trends.series[0].counts.iter().sum::<u64>(), 5);
    assert_eq!(trends.series[1].counts.iter().sum::<u64>(), 6);
    assert_eq!(trends.series[0].counts, [1, 1, 1, 0, 1, 0, 1, 0]);
    assert_eq!(trends.series[1].counts, [2, 0, 1, 1, 0, 1, 0, 1]);

    let csv = archive.export(&query("lantern"), ExportFormat::Csv)?;
    assert!(csv.starts_with("id,date,language,sources,category,title\r\n"));
    assert_eq!(csv.lines().count(), 6);
    Ok(())
}

#[test]
fn write_is_idempotent_and_replaces_stale_content() -> anyhow::Result<()> {
    let runtime = tempfile::tempdir()?;
    std::fs::create_dir_all(runtime.path().join("data/search_index"))?;
    write(runtime.path())?;
    let directory = runtime.path().join("data/kcna_archive");
    std::fs::write(directory.join("stale.txt"), "left by an earlier stage")?;
    write(runtime.path())?;
    assert!(!directory.join("stale.txt").exists());
    assert_eq!(
        Archive::load_into_memory(&directory)?.status()?.indexed,
        ENTRIES.len() as u64
    );
    assert_eq!(document_id().as_u128(), ID_BASE + 1);
    Ok(())
}

#[test]
fn write_refuses_symlinked_destinations() -> anyhow::Result<()> {
    let elsewhere = tempfile::tempdir()?;
    let runtime = tempfile::tempdir()?;
    std::fs::create_dir(runtime.path().join("data"))?;
    symlink(elsewhere.path(), runtime.path().join("data/kcna_archive"))?;
    assert!(write(runtime.path()).is_err());
    assert_eq!(std::fs::read_dir(elsewhere.path())?.count(), 0);

    let other = tempfile::tempdir()?;
    symlink(elsewhere.path(), other.path().join("data"))?;
    assert!(write(other.path()).is_err());
    assert_eq!(std::fs::read_dir(elsewhere.path())?.count(), 0);
    Ok(())
}
