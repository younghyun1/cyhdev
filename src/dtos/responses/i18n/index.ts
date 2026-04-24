// i18n response DTOs

// Expand with actual fields as your API responses define them.
export interface GetCountryLanguageBundleResponse {
  // Example placeholder field:
  // bundles: CountryLanguageBundle[];
}

export interface UiTextBundleResponse {
  locale: string;
  fallback_locale: string;
  texts: Record<string, string>;
}
