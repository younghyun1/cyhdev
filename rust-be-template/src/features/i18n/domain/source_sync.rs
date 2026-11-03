//! Selection of embedded UI text rows whose stored copy is missing or stale.

use std::collections::HashMap;

use super::source::UiTextSourceBundle;

/// A stored file-backed row (no subdivision), identified by key and locale codes.
pub struct StoredUiText {
    pub reference_key: String,
    pub country_code: i32,
    pub language_code: i32,
    pub content: String,
}

/// An embedded row that must be written because PostgreSQL lacks it or holds other content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiTextSourceRow<'a> {
    pub reference_key: &'a str,
    pub country_code: i32,
    pub language_code: i32,
    pub content: &'a str,
}

/// Counts reported by one file-backed source synchronization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSyncOutcome {
    /// Embedded rows compared with PostgreSQL.
    pub source_rows: usize,
    /// Rows inserted or updated; zero when the catalogs match the database.
    pub written_rows: usize,
}

/// Returns embedded rows absent from `stored` or differing from it, in source order.
///
/// Rows are matched per locale, so a key stored only for another locale still
/// counts as missing. Stored rows without an embedded counterpart are ignored.
pub fn changed_source_rows<'a>(
    bundles: &'a [UiTextSourceBundle],
    stored: &[StoredUiText],
) -> Vec<UiTextSourceRow<'a>> {
    let stored_content = stored
        .iter()
        .map(|row| {
            (
                (
                    row.reference_key.as_str(),
                    row.country_code,
                    row.language_code,
                ),
                row.content.as_str(),
            )
        })
        .collect::<HashMap<_, _>>();
    bundles
        .iter()
        .flat_map(|bundle| {
            let country_code = bundle.locale.country_code();
            let language_code = bundle.locale.language_code();
            bundle.entries.iter().map(move |entry| UiTextSourceRow {
                reference_key: &entry.key,
                country_code,
                language_code,
                content: &entry.content,
            })
        })
        .filter(|row| {
            stored_content.get(&(row.reference_key, row.country_code, row.language_code))
                != Some(&row.content)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{StoredUiText, UiTextSourceRow, changed_source_rows};
    use crate::features::i18n::domain::{
        locale::UiLocale,
        source::{UiTextSourceBundle, UiTextSourceEntry},
    };

    fn bundle(locale: UiLocale, entries: &[(&str, &str)]) -> UiTextSourceBundle {
        UiTextSourceBundle {
            locale,
            entries: entries
                .iter()
                .map(|(key, content)| UiTextSourceEntry {
                    key: (*key).to_owned(),
                    content: (*content).to_owned(),
                })
                .collect(),
        }
    }

    fn stored(locale: UiLocale, key: &str, content: &str) -> StoredUiText {
        StoredUiText {
            reference_key: key.to_owned(),
            country_code: locale.country_code(),
            language_code: locale.language_code(),
            content: content.to_owned(),
        }
    }

    #[test]
    fn unchanged_catalogs_select_nothing() {
        let bundles = [
            bundle(
                UiLocale::EnUs,
                &[("common.save", "Save"), ("common.cancel", "Cancel")],
            ),
            bundle(UiLocale::KoKr, &[("common.save", "저장")]),
        ];
        let rows = [
            stored(UiLocale::KoKr, "common.save", "저장"),
            stored(UiLocale::EnUs, "common.cancel", "Cancel"),
            stored(UiLocale::EnUs, "common.save", "Save"),
            // Rows outside the catalogs never force a write.
            stored(UiLocale::EnUs, "legacy.key", "Legacy"),
        ];
        assert!(changed_source_rows(&bundles, &rows).is_empty());
    }

    #[test]
    fn missing_and_drifted_rows_are_selected_in_source_order() {
        let bundles = [
            bundle(
                UiLocale::EnUs,
                &[("common.save", "Save"), ("common.cancel", "Cancel")],
            ),
            bundle(UiLocale::KoKr, &[("common.save", "저장")]),
        ];
        // English "save" drifted, English "cancel" is current, and Korean
        // "save" exists only under another locale's codes.
        let rows = [
            stored(UiLocale::EnUs, "common.save", "Drifted"),
            stored(UiLocale::EnUs, "common.cancel", "Cancel"),
            stored(UiLocale::JaJp, "common.save", "저장"),
        ];
        assert_eq!(
            changed_source_rows(&bundles, &rows),
            [
                UiTextSourceRow {
                    reference_key: "common.save",
                    country_code: UiLocale::EnUs.country_code(),
                    language_code: UiLocale::EnUs.language_code(),
                    content: "Save",
                },
                UiTextSourceRow {
                    reference_key: "common.save",
                    country_code: UiLocale::KoKr.country_code(),
                    language_code: UiLocale::KoKr.language_code(),
                    content: "저장",
                },
            ]
        );
    }

    #[test]
    fn an_empty_database_selects_every_embedded_row() {
        let bundles = [bundle(
            UiLocale::DeDe,
            &[("common.save", "Speichern"), ("common.cancel", "Abbrechen")],
        )];
        assert_eq!(changed_source_rows(&bundles, &[]).len(), 2);
    }
}
