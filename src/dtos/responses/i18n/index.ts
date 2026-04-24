export interface UiTextBundleResponse {
  locale: string;
  fallback_locale: string;
  texts: Record<string, string>;
}
