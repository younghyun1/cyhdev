//! ISO 3166 country and subdivision values.

use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct IsoCountry {
    pub country_code: i32,
    pub country_alpha2: String,
    pub country_alpha3: String,
    pub country_eng_name: String,
    pub country_currency: i32,
    pub phone_prefix: String,
    pub country_flag: String,
    pub is_country: bool,
    pub country_primary_language: i32,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct IsoCountrySubdivision {
    pub subdivision_id: i32,
    pub country_code: i32,
    pub subdivision_code: String,
    pub subdivision_name: String,
    pub subdivision_type: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct CountryAndSubdivisions {
    pub country: IsoCountry,
    pub subdivisions: Vec<IsoCountrySubdivision>,
}
