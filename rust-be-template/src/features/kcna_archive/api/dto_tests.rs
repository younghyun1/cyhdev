//! Contract tests: enum spellings, camelCase fields, and conversions.

use kcna_archive_core::{
    service::models::{
        FacetGroups, FacetValue, HistogramResponse, SearchItem, SearchResponse, TextSegment,
    },
    types::{Language, PublicationPrecision, Source},
};
use serde_json::{Value, json};

use super::{
    dto::{
        ArchiveHistogramResponse, ArchiveLanguage, ArchivePrecision, ArchiveSearchResponse,
        ArchiveSource,
    },
    status_dto::{ArchiveServiceState, ArchiveStatusResponse},
};
use crate::features::kcna_archive::{
    error::ArchiveError,
    service::state::{ArchiveStatus, ServiceState},
};

fn json_of<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

#[test]
fn enums_serialize_exactly_as_the_core_does() {
    for language in Language::ALL {
        assert_eq!(
            json_of(&ArchiveLanguage::from(language)),
            json_of(&language)
        );
    }
    for source in Source::ALL {
        assert_eq!(json_of(&ArchiveSource::from(source)), json_of(&source));
    }
    for precision in PublicationPrecision::ALL {
        assert_eq!(
            json_of(&ArchivePrecision::from(precision)),
            json_of(&precision)
        );
    }
}

#[test]
fn a_search_response_converts_with_camel_case_fields() {
    let core = SearchResponse {
        total: 7,
        offset: 0,
        limit: 50,
        items: vec![SearchItem {
            id: "id-1".to_owned(),
            title: vec![TextSegment {
                text: "Peace".to_owned(),
                hit: true,
            }],
            snippet: Vec::new(),
            publication_date: "2026-11-20".to_owned(),
            language: Language::Korean,
            sources: vec![Source::KcnaKp, Source::StilesArchive],
            category: None,
            has_full_text: true,
        }],
        facets: FacetGroups {
            languages: vec![FacetValue {
                value: Language::Korean,
                count: 7,
            }],
            ..FacetGroups::default()
        },
    };
    let value = json_of(&ArchiveSearchResponse::from(core));
    assert_eq!(value["total"], 7);
    assert_eq!(
        value["items"][0],
        json!({
            "id": "id-1",
            "title": [{"text": "Peace", "hit": true}],
            "snippet": [],
            "publicationDate": "2026-11-20",
            "language": "kor",
            "sources": ["kcna_kp", "stiles_archive"],
            "category": null,
            "hasFullText": true
        })
    );
    assert_eq!(
        value["facets"]["languages"],
        json!([{"value": "kor", "count": 7}])
    );
}

#[test]
fn an_unknown_bucket_name_is_an_internal_failure() {
    let converted = ArchiveHistogramResponse::try_from(HistogramResponse {
        bucket: "decade",
        buckets: Vec::new(),
    });
    assert!(matches!(converted, Err(ArchiveError::Internal)));
    let year = ArchiveHistogramResponse::try_from(HistogramResponse {
        bucket: "year",
        buckets: Vec::new(),
    });
    assert_eq!(
        year.map(|response| json_of(&response)).ok(),
        Some(json!({"bucket": "year", "buckets": []}))
    );
}

#[test]
fn the_status_names_every_state_without_a_snapshot() {
    for (state, name) in [
        (ServiceState::NotInstalled, "not_installed"),
        (ServiceState::Loading, "loading"),
        (ServiceState::Failed, "failed"),
    ] {
        let response = ArchiveStatusResponse::from(ArchiveStatus {
            state,
            reloading: false,
            last_reload_failed: false,
            snapshot: None,
        });
        assert_eq!(
            json_of(&response),
            json!({
                "state": name,
                "reloading": false,
                "lastReloadFailed": false,
                "snapshot": null
            })
        );
    }
    assert_eq!(json_of(&ArchiveServiceState::Ready), json!("ready"));
}
