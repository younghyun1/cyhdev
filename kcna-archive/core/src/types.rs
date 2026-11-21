//! Closed value sets of the corpus, shared by the importer, the index, and the API.

use serde::{Deserialize, Serialize};

/// PostgreSQL enum types the corpus values map to in the collector's database.
#[cfg(feature = "diesel")]
pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kcna_source"))]
    pub struct KcnaSource;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kcna_language"))]
    pub struct KcnaLanguage;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kcna_publication_precision"))]
    pub struct KcnaPublicationPrecision;
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "diesel", derive(diesel_derive_enum::DbEnum))]
#[cfg_attr(
    feature = "diesel",
    ExistingTypePath = "crate::types::sql_types::KcnaSource"
)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    KcnaKp,
    KcnaCoJp,
    StilesArchive,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "diesel", derive(diesel_derive_enum::DbEnum))]
#[cfg_attr(
    feature = "diesel",
    ExistingTypePath = "crate::types::sql_types::KcnaLanguage"
)]
#[cfg_attr(feature = "diesel", DbValueStyle = "verbatim")]
pub enum Language {
    #[cfg_attr(feature = "diesel", db_rename = "kor")]
    #[serde(rename = "kor")]
    Korean,
    #[cfg_attr(feature = "diesel", db_rename = "eng")]
    #[serde(rename = "eng")]
    English,
    #[cfg_attr(feature = "diesel", db_rename = "zho")]
    #[serde(rename = "zho")]
    Chinese,
    #[cfg_attr(feature = "diesel", db_rename = "rus")]
    #[serde(rename = "rus")]
    Russian,
    #[cfg_attr(feature = "diesel", db_rename = "spa")]
    #[serde(rename = "spa")]
    Spanish,
    #[cfg_attr(feature = "diesel", db_rename = "jpn")]
    #[serde(rename = "jpn")]
    Japanese,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "diesel", derive(diesel_derive_enum::DbEnum))]
#[cfg_attr(
    feature = "diesel",
    ExistingTypePath = "crate::types::sql_types::KcnaPublicationPrecision"
)]
#[serde(rename_all = "snake_case")]
pub enum PublicationPrecision {
    Second,
    Minute,
    Day,
    InferredDay,
}

impl Language {
    pub const ALL: [Self; 6] = [
        Self::Korean,
        Self::English,
        Self::Chinese,
        Self::Russian,
        Self::Spanish,
        Self::Japanese,
    ];

    /// ISO 639-3 code, as stored and as used in the API.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Korean => "kor",
            Self::English => "eng",
            Self::Chinese => "zho",
            Self::Russian => "rus",
            Self::Spanish => "spa",
            Self::Japanese => "jpn",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }
}

impl PublicationPrecision {
    pub const ALL: [Self; 4] = [Self::Second, Self::Minute, Self::Day, Self::InferredDay];

    /// Identifier as stored and as used in the API.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Day => "day",
            Self::InferredDay => "inferred_day",
        }
    }
}

impl Source {
    pub const ALL: [Self; 3] = [Self::KcnaKp, Self::KcnaCoJp, Self::StilesArchive];

    /// Identifier as stored and as used in the API.
    pub const fn code(self) -> &'static str {
        match self {
            Self::KcnaKp => "kcna_kp",
            Self::KcnaCoJp => "kcna_co_jp",
            Self::StilesArchive => "stiles_archive",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|source| source.code() == code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_the_serialized_names() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert_eq!(
                serde_json::to_value(language).ok(),
                Some(serde_json::Value::String(language.code().to_owned()))
            );
        }
        for source in Source::ALL {
            assert_eq!(Source::from_code(source.code()), Some(source));
            assert_eq!(
                serde_json::to_value(source).ok(),
                Some(serde_json::Value::String(source.code().to_owned()))
            );
        }
        for precision in PublicationPrecision::ALL {
            assert_eq!(
                serde_json::to_value(precision).ok(),
                Some(serde_json::Value::String(precision.code().to_owned()))
            );
        }
        assert_eq!(Language::from_code("xxx"), None);
    }
}
