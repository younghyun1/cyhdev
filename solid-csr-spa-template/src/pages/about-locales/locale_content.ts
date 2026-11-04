import type { UiLocale } from "../../i18n/keys";

type ContentModule<T> = { readonly default: T };

export type LocaleContentLoaders<T> = Readonly<
  Record<UiLocale, () => Promise<ContentModule<T>>>
>;

/** Content together with the locale it is actually written in. */
export type LocalizedContent<T> = {
  readonly locale: UiLocale;
  readonly content: T;
};

/**
 * Returns a loader that downloads one locale's content chunk on first use, so
 * a visitor only fetches the active language. Promises are memoized per
 * locale; the map can hold at most one entry per supported locale. A chunk that
 * fails to load, for example after a deployment removed it, resolves to
 * English and reports `en-US` so the page's `lang` stays truthful. A failed
 * English load is not memoized, so a later render can retry it, and still
 * rejects to reach the route's error boundary.
 */
export function createLocaleContentLoader<T>(
  loaders: LocaleContentLoaders<T>,
): (tag: UiLocale) => Promise<LocalizedContent<T>> {
  const pending = new Map<UiLocale, Promise<LocalizedContent<T>>>();
  const loadEnglish = (): Promise<LocalizedContent<T>> => {
    const cached = pending.get("en-US");
    if (cached) return cached;
    const request = loaders["en-US"]().then(
      (module) => ({ locale: "en-US" as const, content: module.default }),
      (error: unknown) => {
        pending.delete("en-US");
        throw error;
      },
    );
    pending.set("en-US", request);
    return request;
  };

  return (tag) => {
    if (tag === "en-US") return loadEnglish();
    const cached = pending.get(tag);
    if (cached) return cached;
    const request = loaders[tag]().then(
      (module) => ({ locale: tag, content: module.default }),
      (error: unknown) => {
        pending.delete(tag);
        console.warn(`Falling back to English; ${tag} content failed to load`, error);
        return loadEnglish();
      },
    );
    pending.set(tag, request);
    return request;
  };
}
