export interface GetCountryLanguageBundleRequest {
  country_code: number;
  language_code: number;
}

export interface GetUiTextBundleRequest {
  locale?: string;
}
