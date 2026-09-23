import { type Component, lazy } from "solid-js";
import type { UiLocale } from "../../i18n/keys";

type PageModule = { readonly default: Component };

export type LocalePageLoaders = Readonly<
  Record<UiLocale, () => Promise<PageModule>>
>;

/**
 * One lazy component per locale, so a visitor downloads only the active
 * language's page chunk instead of all eight. A chunk that fails to load, for
 * example after a deployment removed it, falls back to the English page rather
 * than breaking the route; an English failure still reaches the route's error
 * boundary. While a newly selected language loads, the enclosing Loading
 * boundary keeps showing the previous language.
 */
export function lazyLocalePages(
  loaders: LocalePageLoaders,
): Readonly<Record<UiLocale, Component>> {
  const load = (tag: UiLocale) => (): Promise<PageModule> =>
    loaders[tag]().catch((error: unknown) => {
      if (tag === "en-US") throw error;
      console.warn(`Falling back to English; ${tag} page failed to load`, error);
      return loaders["en-US"]();
    });
  const pages = {} as Record<UiLocale, Component>;
  for (const tag of Object.keys(loaders) as UiLocale[]) {
    pages[tag] = lazy(load(tag));
  }
  return pages;
}
