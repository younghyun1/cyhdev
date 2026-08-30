//! Fixed country/subdivision catalog and narrow flag lookup port.

use std::{collections::HashMap, sync::Arc};

use serde_derive::Serialize;

use super::country::{CountryAndSubdivisions, IsoCountry, IsoCountrySubdivision};

pub trait CountryFlagLookup {
    fn flag_for_country_code(&self, country_code: i32) -> Option<&str>;
}

#[derive(Serialize)]
pub struct CountryAndSubdivisionsTable {
    rows: Vec<CountryAndSubdivisions>,
    by_id: HashMap<i32, usize>,
    by_country_alpha2: HashMap<String, usize>,
    by_country_alpha3: HashMap<String, usize>,
    serialized_country_list: Arc<serde_json::Value>,
}

impl CountryAndSubdivisionsTable {
    pub fn new(countries: Vec<IsoCountry>, subdivisions: Vec<IsoCountrySubdivision>) -> Self {
        let mut countries_by_code = countries
            .into_iter()
            .map(|country| (country.country_code, country))
            .collect::<HashMap<_, _>>();
        let mut subdivisions_by_country = HashMap::<i32, Vec<IsoCountrySubdivision>>::new();
        for subdivision in subdivisions {
            subdivisions_by_country
                .entry(subdivision.country_code)
                .or_default()
                .push(subdivision);
        }
        let mut rows = countries_by_code
            .drain()
            .map(|(country_code, country)| CountryAndSubdivisions {
                country,
                subdivisions: subdivisions_by_country
                    .remove(&country_code)
                    .unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            left.country.country_eng_name.cmp(&right.country.country_eng_name)
        });
        let mut by_id = HashMap::with_capacity(rows.len());
        let mut by_country_alpha2 = HashMap::with_capacity(rows.len());
        let mut by_country_alpha3 = HashMap::with_capacity(rows.len());
        for (index, combined) in rows.iter().enumerate() {
            by_id.insert(combined.country.country_code, index);
            by_country_alpha2.insert(combined.country.country_alpha2.clone(), index);
            by_country_alpha3.insert(combined.country.country_alpha3.clone(), index);
        }
        let serialized_country_list = Arc::new(serde_json::json!({
            "countries": rows.iter().map(|combined| &combined.country).collect::<Vec<_>>()
        }));
        Self { rows, by_id, by_country_alpha2, by_country_alpha3, serialized_country_list }
    }

    pub fn new_empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn rows(&self) -> &[CountryAndSubdivisions] {
        &self.rows
    }

    pub fn country(&self, country_code: i32) -> Option<&CountryAndSubdivisions> {
        self.by_id
            .get(&country_code)
            .and_then(|index| self.rows.get(*index))
    }

    pub fn lookup_by_alpha2(&self, code: &str) -> Option<&CountryAndSubdivisions> {
        self.by_country_alpha2
            .get(code)
            .and_then(|index| self.rows.get(*index))
    }

    pub fn lookup_by_alpha3(&self, code: &str) -> Option<&CountryAndSubdivisions> {
        self.by_country_alpha3
            .get(code)
            .and_then(|index| self.rows.get(*index))
    }

    pub fn serialized_country_list(&self) -> Arc<serde_json::Value> {
        Arc::clone(&self.serialized_country_list)
    }

    pub fn as_dispatch_json(&self) -> serde_json::Value {
        serde_json::json!({ "countries": self.rows })
    }

    pub fn get_flag_by_code(&self, country_code: i32) -> Option<String> {
        self.flag_for_country_code(country_code).map(str::to_owned)
    }
}

impl CountryFlagLookup for CountryAndSubdivisionsTable {
    fn flag_for_country_code(&self, country_code: i32) -> Option<&str> {
        self.country(country_code)
            .map(|combined| combined.country.country_flag.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{CountryAndSubdivisionsTable, CountryFlagLookup};
    use crate::features::reference_data::domain::country::{
        IsoCountry, IsoCountrySubdivision,
    };

    #[test]
    fn flag_port_and_country_indexes_are_safe() {
        let catalog = CountryAndSubdivisionsTable::new(
            vec![IsoCountry {
                country_code: 840,
                country_alpha2: "US".to_owned(),
                country_alpha3: "USA".to_owned(),
                country_eng_name: "United States".to_owned(),
                country_currency: 840,
                phone_prefix: "+1".to_owned(),
                country_flag: "🇺🇸".to_owned(),
                is_country: true,
                country_primary_language: 41,
            }],
            vec![IsoCountrySubdivision {
                subdivision_id: 1,
                country_code: 840,
                subdivision_code: "US-CO".to_owned(),
                subdivision_name: "Colorado".to_owned(),
                subdivision_type: Some("state".to_owned()),
            }],
        );
        assert_eq!(catalog.flag_for_country_code(840), Some("🇺🇸"));
        assert_eq!(catalog.flag_for_country_code(999), None);
        assert_eq!(
            catalog.country(840).map(|country| country.subdivisions.len()),
            Some(1)
        );
    }
}
