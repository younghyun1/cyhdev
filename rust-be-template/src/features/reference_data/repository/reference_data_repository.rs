//! Diesel persistence for fixed ISO reference tables.

use diesel::{QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::bb8::Pool};

use crate::{
    features::reference_data::domain::{
        country::{IsoCountry, IsoCountrySubdivision},
        currency::IsoCurrency,
        language::IsoLanguage,
    },
    schema::{iso_country, iso_country_subdivision, iso_currency, iso_language},
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = iso_country, check_for_backend(diesel::pg::Pg))]
struct CountryRecord {
    country_code: i32,
    country_alpha2: String,
    country_alpha3: String,
    country_eng_name: String,
    country_currency: i32,
    phone_prefix: String,
    country_flag: String,
    is_country: bool,
    country_primary_language: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = iso_country_subdivision, check_for_backend(diesel::pg::Pg))]
struct SubdivisionRecord {
    subdivision_id: i32,
    country_code: i32,
    subdivision_code: String,
    subdivision_name: String,
    subdivision_type: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = iso_language, check_for_backend(diesel::pg::Pg))]
struct LanguageRecord {
    language_code: i32,
    language_alpha2: String,
    language_alpha3: String,
    language_eng_name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = iso_currency, check_for_backend(diesel::pg::Pg))]
struct CurrencyRecord {
    currency_code: i32,
    currency_alpha3: String,
    currency_name: String,
}

pub struct ReferenceDataRepository {
    pool: Pool<AsyncPgConnection>,
}

impl ReferenceDataRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    pub async fn countries(
        &self,
    ) -> anyhow::Result<(Vec<IsoCountry>, Vec<IsoCountrySubdivision>)> {
        let mut connection = self.pool.get().await?;
        let countries = iso_country::table
            .select(CountryRecord::as_select())
            .load::<CountryRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let subdivisions = iso_country_subdivision::table
            .select(SubdivisionRecord::as_select())
            .load::<SubdivisionRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok((countries, subdivisions))
    }

    pub async fn languages(&self) -> anyhow::Result<Vec<IsoLanguage>> {
        let mut connection = self.pool.get().await?;
        Ok(iso_language::table
            .select(LanguageRecord::as_select())
            .load::<LanguageRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn currencies(&self) -> anyhow::Result<Vec<IsoCurrency>> {
        let mut connection = self.pool.get().await?;
        Ok(iso_currency::table
            .select(CurrencyRecord::as_select())
            .load::<CurrencyRecord>(&mut connection)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

impl From<CountryRecord> for IsoCountry {
    fn from(value: CountryRecord) -> Self {
        Self {
            country_code: value.country_code,
            country_alpha2: value.country_alpha2,
            country_alpha3: value.country_alpha3,
            country_eng_name: value.country_eng_name,
            country_currency: value.country_currency,
            phone_prefix: value.phone_prefix,
            country_flag: value.country_flag,
            is_country: value.is_country,
            country_primary_language: value.country_primary_language,
        }
    }
}

impl From<SubdivisionRecord> for IsoCountrySubdivision {
    fn from(value: SubdivisionRecord) -> Self {
        Self {
            subdivision_id: value.subdivision_id,
            country_code: value.country_code,
            subdivision_code: value.subdivision_code,
            subdivision_name: value.subdivision_name,
            subdivision_type: value.subdivision_type,
        }
    }
}

impl From<LanguageRecord> for IsoLanguage {
    fn from(value: LanguageRecord) -> Self {
        Self {
            language_code: value.language_code,
            language_alpha2: value.language_alpha2,
            language_alpha3: value.language_alpha3,
            language_eng_name: value.language_eng_name,
        }
    }
}

impl From<CurrencyRecord> for IsoCurrency {
    fn from(value: CurrencyRecord) -> Self {
        Self {
            currency_code: value.currency_code,
            currency_alpha3: value.currency_alpha3,
            currency_name: value.currency_name,
        }
    }
}
