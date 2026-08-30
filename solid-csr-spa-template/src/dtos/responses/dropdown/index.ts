// Dropdown response DTOs (countries, languages, subdivisions)

export interface IsoLanguage {
  language_code: number;
  language_alpha2: string;
  language_alpha3: string;
  language_eng_name: string;
}

export interface IsoCountry {
  country_code: number;
  country_alpha2: string;
  country_alpha3: string;
  country_eng_name: string;
  country_currency: number;
  phone_prefix: string;
  country_flag: string;
  is_country: boolean;
  country_primary_language: number;
}

export interface IsoCountrySubdivision {
  subdivision_id: number;
  country_code: number;
  subdivision_code: string;
  subdivision_name: string;
  subdivision_type: string | null;
}

/** Response from GET /api/dropdown/language */
export type GetLanguagesResponse = IsoLanguage[];

/** Response from GET /api/dropdown/language/{language_id} */
export type GetLanguageResponse = IsoLanguage;

/** Response from GET /api/dropdown/country */
export type GetCountriesResponse = { countries: IsoCountry[] };

/** Response from GET /api/dropdown/country/{country_id} */
export type GetCountryResponse = IsoCountry;

/** Response from GET /api/dropdown/country/{country_id}/subdivision */
export type GetSubdivisionsResponse = IsoCountrySubdivision[];
