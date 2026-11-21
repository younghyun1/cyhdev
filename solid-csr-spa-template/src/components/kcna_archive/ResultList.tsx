import { createEffect, For, onSettled, Show, type Component } from "solid-js";

import type { ArchiveSearchItem } from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { languageKey, sourceKey } from "../../utils/kcna_archive/constants";
import { failureKey, type FailureKind } from "../../utils/kcna_archive/failure";
import { formatDate } from "../../utils/kcna_archive/format";
import Segments from "./Segments";

interface ResultListProps {
  readonly items: ReadonlyArray<ArchiveSearchItem>;
  /** Matches the reader may page through, already capped at the server's window. */
  readonly total: number;
  readonly selectedId: string | undefined;
  /** A later page is loading. */
  readonly loadingMore: boolean;
  /** Why loading a later page failed, if it did. */
  readonly failure: FailureKind | undefined;
  readonly hrefFor: (item: ArchiveSearchItem) => string;
  /** The result opens beside the list, so following it must not move the page. */
  readonly keepScroll: boolean;
  readonly onLoadMore: () => void;
}

const ResultList: Component<ResultListProps> = (props) => {
  let sentinel: HTMLDivElement | undefined;
  let observer: IntersectionObserver | undefined;

  const hasMore = (): boolean => props.items.length < props.total;
  // Loads automatically only while healthy: after a failure the visible retry
  // button is the single trigger, so an outage cannot cause a request loop.
  const wantsMore = (): boolean => hasMore() && !props.loadingMore && props.failure === undefined;

  onSettled(() => {
    if (typeof IntersectionObserver === "undefined") return;
    observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting) && wantsMore()) props.onLoadMore();
      },
      { rootMargin: "600px 0px" },
    );
    if (sentinel !== undefined) observer.observe(sentinel);
    return () => observer?.disconnect();
  });

  // An observer reports changes only. Observing again after each page makes it
  // report the current state, which keeps loading while the sentinel is still
  // near the viewport after a short page.
  createEffect(
    () => [props.items.length, props.loadingMore] as const,
    () => {
      if (observer === undefined || sentinel === undefined) return;
      observer.unobserve(sentinel);
      observer.observe(sentinel);
    },
  );

  const sources = (item: ArchiveSearchItem): string =>
    item.sources.map((source) => t(sourceKey(source))).join(", ");

  return (
    <>
      <ol class="kcna-rows">
        <For each={props.items}>
          {(item) => (
            <li data-id={item.id} class="kcna-row-item">
              <a
                class="kcna-row"
                href={props.hrefFor(item)}
                noscroll={props.keepScroll}
                aria-current={props.selectedId === item.id ? "true" : undefined}
              >
                <span class="kcna-row-meta">
                  <time datetime={item.publicationDate}>{formatDate(item.publicationDate, locale())}</time>
                  <span class="kcna-tag" dir="ltr" title={t(languageKey(item.language))}>
                    {item.language.toUpperCase()}
                  </span>
                  <span class="kcna-row-sources">{sources(item)}</span>
                  <Show when={item.category}>
                    {(category) => (
                      <span class="kcna-row-category" lang={item.language} dir="ltr">
                        {category()}
                      </span>
                    )}
                  </Show>
                </span>
                <span class="kcna-row-title" lang={item.language} dir="ltr">
                  <Segments segments={item.title} />
                </span>
                <Show when={item.snippet.length > 0}>
                  <span class="kcna-row-snippet" lang={item.language} dir="ltr">
                    <Segments segments={item.snippet} />
                  </span>
                </Show>
              </a>
            </li>
          )}
        </For>
      </ol>
      <Show when={props.total === 0}>
        <p class="kcna-empty">{t("kcna.results.empty")}</p>
      </Show>
      <Show when={props.failure}>
        {(failure) => (
          <p class="kcna-notice kcna-notice--error" role="alert">
            <span class="kcna-notice-text">{t(failureKey(failure()))}</span>
            <button type="button" class="kcna-button" onClick={() => props.onLoadMore()}>
              {t("kcna.state.retry")}
            </button>
          </p>
        )}
      </Show>
      <div ref={sentinel} class="kcna-sentinel" aria-live="polite">
        <Show when={props.loadingMore}>
          <span class="kcna-spinner" role="status" aria-label={t("common.loading")} />
        </Show>
        <Show when={wantsMore()}>
          <button type="button" class="kcna-button" onClick={() => props.onLoadMore()}>
            {t("kcna.results.more")}
          </button>
        </Show>
      </div>
    </>
  );
};

export default ResultList;
