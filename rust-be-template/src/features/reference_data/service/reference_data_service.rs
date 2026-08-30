//! Fixed ISO catalog synchronization and lookup use cases.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tracing::{error, info};

use crate::features::reference_data::{
    domain::{
        catalog::{CountryAndSubdivisionsTable, CountryFlagLookup},
        country::{CountryAndSubdivisions, IsoCountrySubdivision},
        currency::IsoCurrencyTable,
        language::{IsoLanguage, IsoLanguageTable},
    },
    repository::reference_data_repository::ReferenceDataRepository,
};

pub struct ReferenceDataService {
    repository: Arc<ReferenceDataRepository>,
    countries: RwLock<CountryAndSubdivisionsTable>,
    languages: RwLock<IsoLanguageTable>,
    currencies: RwLock<IsoCurrencyTable>,
}

impl ReferenceDataService {
    pub fn new(repository: Arc<ReferenceDataRepository>) -> Self {
        Self {
            repository,
            countries: RwLock::new(CountryAndSubdivisionsTable::new_empty()),
            languages: RwLock::new(IsoLanguageTable::new_empty()),
            currencies: RwLock::new(IsoCurrencyTable::new_empty()),
        }
    }

    pub async fn synchronize(&self) -> anyhow::Result<()> {
        let start = tokio::time::Instant::now();
        let (country_result, language_result, currency_result) = tokio::join!(
            self.repository.countries(),
            self.repository.languages(),
            self.repository.currencies(),
        );
        match country_result {
            Ok((countries, subdivisions)) => {
                let rows = countries.len().saturating_add(subdivisions.len());
                *self.countries.write().await =
                    CountryAndSubdivisionsTable::new(countries, subdivisions);
                info!(rows_synchronized = rows, "Synchronized country reference data");
            }
            Err(source) => error!(error = %source, "Failed to synchronize country data"),
        }
        match language_result {
            Ok(languages) => {
                let rows = languages.len();
                *self.languages.write().await = IsoLanguageTable::from(languages);
                info!(rows_synchronized = rows, "Synchronized language reference data");
            }
            Err(source) => error!(error = %source, "Failed to synchronize language data"),
        }
        match currency_result {
            Ok(currencies) => {
                let rows = currencies.len();
                *self.currencies.write().await = IsoCurrencyTable::from(currencies);
                info!(rows_synchronized = rows, "Synchronized currency reference data");
            }
            Err(source) => error!(error = %source, "Failed to synchronize currency data"),
        }
        info!(elapsed = ?start.elapsed(), "Reference-data synchronization completed");
        Ok(())
    }

    pub async fn countries_json(&self) -> Arc<serde_json::Value> {
        self.countries.read().await.serialized_country_list()
    }

    pub async fn country(&self, code: i32) -> Option<CountryAndSubdivisions> {
        self.countries.read().await.country(code).cloned()
    }

    pub async fn country_by_alpha2(&self, code: &str) -> Option<CountryAndSubdivisions> {
        self.countries.read().await.lookup_by_alpha2(code).cloned()
    }

    pub async fn country_by_alpha3(&self, code: &str) -> Option<CountryAndSubdivisions> {
        self.countries.read().await.lookup_by_alpha3(code).cloned()
    }

    pub async fn subdivisions(&self, code: i32) -> Option<Vec<IsoCountrySubdivision>> {
        self.countries
            .read()
            .await
            .country(code)
            .map(|country| country.subdivisions.clone())
    }

    pub async fn languages(&self) -> Vec<IsoLanguage> {
        self.languages.read().await.rows().to_vec()
    }

    pub async fn language(&self, code: i32) -> Option<IsoLanguage> {
        self.languages.read().await.lookup_by_code(code)
    }

    pub async fn country_flag(&self, code: i32) -> Option<String> {
        self.countries
            .read()
            .await
            .flag_for_country_code(code)
            .map(str::to_owned)
    }

    pub async fn country_flags(&self, codes: &[i32]) -> HashMap<i32, String> {
        let countries = self.countries.read().await;
        codes
            .iter()
            .copied()
            .filter_map(|code| {
                countries
                    .flag_for_country_code(code)
                    .map(|flag| (code, flag.to_owned()))
            })
            .collect()
    }
}

#[async_trait::async_trait]
pub trait CountryFlagLookupPort: Send + Sync {
    async fn country_flag(&self, country_code: i32) -> Option<String>;
    async fn country_flags(&self, country_codes: &[i32]) -> HashMap<i32, String>;
}

#[async_trait::async_trait]
impl CountryFlagLookupPort for ReferenceDataService {
    async fn country_flag(&self, country_code: i32) -> Option<String> {
        ReferenceDataService::country_flag(self, country_code).await
    }

    async fn country_flags(&self, country_codes: &[i32]) -> HashMap<i32, String> {
        ReferenceDataService::country_flags(self, country_codes).await
    }
}
