import { type Component, Show, lazy } from "solid-js";
import type { UiLocale } from "../../i18n/keys";
import { locale } from "../../state/i18n";

type PageModule = { readonly default: Component };

export type LocalePageLoaders = Readonly<
  Record<UiLocale, () => Promise<PageModule>>
>;

/**
 * One lazy component per locale, so a visitor downloads only the active
 * language's page chunk instead of all eight. A chunk that fails to load, for
 * example after a deployment removed it, falls back to the English page rather
 * than breaking the route; an English failure still reaches the route's error
 * boundary.
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

/**
 * Page component that renders the current UI locale's lazy page. While a newly
 * selected language loads, the enclosing Loading boundary keeps showing the
 * previous one. A keyed Show is used instead of Dynamic, which would pull its
 * element-creation code into the shared initial chunk.
 */
export function createLocalePage(loaders: LocalePageLoaders): Component {
  const pages = lazyLocalePages(loaders);
  return function LocalePage() {
    return (
      <Show when={locale()} keyed>
        {(tag) => {
          const Page = pages[tag];
          return <Page />;
        }}
      </Show>
    );
  };
}
