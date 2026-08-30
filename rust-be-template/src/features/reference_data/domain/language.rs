//! ISO 639 language values and fixed lookup indexes.

use std::collections::{BTreeMap, HashMap};

use serde_derive::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct IsoLanguage {
    pub language_code: i32,
    pub language_alpha2: String,
    pub language_alpha3: String,
    pub language_eng_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct TruncatedLanguage {
    pub language_alpha2: String,
    pub language_alpha3: String,
    pub language_eng_name: String,
}

#[derive(Serialize)]
pub struct IsoLanguageTable {
    rows: Vec<IsoLanguage>,
    by_code: HashMap<i32, usize>,
    by_alpha2: HashMap<String, usize>,
    by_alpha3: HashMap<String, usize>,
    pub serialized_map: serde_json::Value,
}

impl From<Vec<IsoLanguage>> for IsoLanguageTable {
    fn from(rows: Vec<IsoLanguage>) -> Self {
        let mut by_code = HashMap::with_capacity(rows.len());
        let mut by_alpha2 = HashMap::with_capacity(rows.len());
        let mut by_alpha3 = HashMap::with_capacity(rows.len());
        let mut languages = BTreeMap::<i32, TruncatedLanguage>::new();
        for (index, row) in rows.iter().enumerate() {
            by_code.insert(row.language_code, index);
            by_alpha2.insert(row.language_alpha2.clone(), index);
            by_alpha3.insert(row.language_alpha3.clone(), index);
            languages.insert(
                row.language_code,
                TruncatedLanguage {
                    language_alpha2: row.language_alpha2.clone(),
                    language_alpha3: row.language_alpha3.clone(),
                    language_eng_name: row.language_eng_name.clone(),
                },
            );
        }
        let serialized_map = match serde_json::to_value(&languages) {
            Ok(value) => value,
            Err(error) => {
                error!(error = %error, "Failed to serialize language reference cache");
                serde_json::Value::Null
            }
        };
        Self {
            rows,
            by_code,
            by_alpha2,
            by_alpha3,
            serialized_map,
        }
    }
}

impl IsoLanguageTable {
    pub fn new_empty() -> Self {
        Self::from(Vec::new())
    }

    pub fn rows(&self) -> &[IsoLanguage] {
        &self.rows
    }

    pub fn lookup_by_code(&self, code: i32) -> Option<IsoLanguage> {
        self.by_code
            .get(&code)
            .and_then(|index| self.rows.get(*index))
            .cloned()
    }

    pub fn lookup_by_alpha2(&self, code: &str) -> Option<IsoLanguage> {
        self.by_alpha2
            .get(code)
            .and_then(|index| self.rows.get(*index))
            .cloned()
    }

    pub fn lookup_by_alpha3(&self, code: &str) -> Option<IsoLanguage> {
        self.by_alpha3
            .get(code)
            .and_then(|index| self.rows.get(*index))
            .cloned()
    }
}
