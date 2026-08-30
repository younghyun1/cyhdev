//! ISO 4217 currency values and fixed lookup indexes.

use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct IsoCurrency {
    pub currency_code: i32,
    pub currency_alpha3: String,
    pub currency_name: String,
}

#[derive(Serialize)]
pub struct IsoCurrencyTable {
    rows: Vec<IsoCurrency>,
    by_code: HashMap<i32, usize>,
    by_alpha3: HashMap<String, usize>,
}

impl From<Vec<IsoCurrency>> for IsoCurrencyTable {
    fn from(rows: Vec<IsoCurrency>) -> Self {
        let mut by_code = HashMap::with_capacity(rows.len());
        let mut by_alpha3 = HashMap::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            by_code.insert(row.currency_code, index);
            by_alpha3.insert(row.currency_alpha3.clone(), index);
        }
        Self { rows, by_code, by_alpha3 }
    }
}

impl IsoCurrencyTable {
    pub fn new_empty() -> Self {
        Self::from(Vec::new())
    }

    pub fn lookup_by_code(&self, code: i32) -> Option<IsoCurrency> {
        self.by_code
            .get(&code)
            .and_then(|index| self.rows.get(*index))
            .cloned()
    }

    pub fn lookup_by_alpha3(&self, code: &str) -> Option<IsoCurrency> {
        self.by_alpha3
            .get(code)
            .and_then(|index| self.rows.get(*index))
            .cloned()
    }
}
