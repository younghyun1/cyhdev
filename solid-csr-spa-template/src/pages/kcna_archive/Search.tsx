import {
  createEffect,
  createMemo,
  createSignal,
  Errored,
  isPending,
  Loading,
  refresh,
  Show,
  type Component,
} from "solid-js";
import KcnaArchiveLayout from "./Layout";
import { useNavigate, useSearchParams } from "@solidjs/router";

import FilterPanel from "../../components/kcna_archive/FilterPanel";
import Histogram from "../../components/kcna_archive/Histogram";
import { ChartSkeleton, DocumentSkeleton, Failure, ListSkeleton } from "../../components/kcna_archive/Notice";
import Reader from "../../components/kcna_archive/Reader";
import ResultList from "../../components/kcna_archive/ResultList";
import SearchForm from "../../components/kcna_archive/SearchForm";
import { focusPrimaryInput, useArchiveShortcuts } from "../../components/kcna_archive/useArchiveShortcuts";
import { kcnaArchiveApi, type ArchiveSearchItem } from "../../services/contracts/kcna_archive";
import { locale, t, tx } from "../../state/i18n";
import { createMediaQuery } from "../../utils/mediaQuery";
import { RESULT_WINDOW } from "../../utils/kcna_archive/constants";
import { failureKind } from "../../utils/kcna_archive/failure";
import { formatCount, formatDate } from "../../utils/kcna_archive/format";
import {
  EMPTY_QUERY,
  firstParam,
  parseQuery,
  queryFromString,
  queryParams,
  queryString,
  withoutRange,
  type ArchiveQuery,
} from "../../utils/kcna_archive/query";
import { besideLanguageOf, documentHref, searchHref } from "../../utils/kcna_archive/routes";
import {
  createRequestGate,
  nextOffset,
  noLaterPages,
  pageFailed,
  pageReceived,
  pageRequested,
  pagesFor,
  type LaterPages,
} from "../../utils/kcna_archive/sequence";

/** Below this width the reader moves to its own page. */
const STACKED = "(max-width: 1023px)";

const KcnaArchiveSearch: Component = () => {
  const navigate = useNavigate();
  const [params, setParams] = useSearchParams();
  const stacked = createMediaQuery(STACKED);

  // Keys are plain strings, so a memo only changes when its part of the URL
  // really changed: opening a document does not refetch the results.
  const queryKey = createMemo(() => queryString(parseQuery(params)));
  const query = createMemo(() => queryFromString(queryKey()));
  const histogramKey = createMemo(() => queryString(withoutRange(query())));
  const documentId = createMemo(() => firstParam(params.doc));
  const besideLanguage = createMemo(() => besideLanguageOf(params.with));

  // Each read is its own async memo, and a newer key supersedes an older
  // response, so a slow answer for a query the reader left is never shown.
  const firstPage = createMemo(async () => kcnaArchiveApi.search(queryFromString(queryKey()), 0));
  const histogram = createMemo(async () =>
    kcnaArchiveApi.histogram(queryFromString(histogramKey()), "month"),
  );
  const opened = createMemo(async () => {
    const id = documentId();
    const text = query().text;
    // The reader is not shown while stacked, so nothing is fetched for it.
    return id === undefined || stacked() ? null : kcnaArchiveApi.document(id, text);
  });
  const beside = createMemo(async () => {
    const language = besideLanguage();
    const text = query().text;
    const link = opened()?.translations.find((translation) => translation.language === language);
    return link === undefined ? null : kcnaArchiveApi.document(link.id, text);
  });

  // Later pages are appended by hand; the gate drops a page that arrives after
  // the query changed.
  const gate = createRequestGate();
  const [later, setLater] = createSignal<LaterPages<ArchiveSearchItem>>(noLaterPages());
  createEffect(queryKey, () => {
    gate.invalidate();
  });
  const laterPages = (): LaterPages<ArchiveSearchItem> => pagesFor(later(), queryKey());
  const items = (): ReadonlyArray<ArchiveSearchItem> => [...firstPage().items, ...laterPages().items];

  const loadMore = async (): Promise<void> => {
    const key = queryKey();
    const current = laterPages();
    const offset = nextOffset(current);
    if (current.loading || offset === undefined) return;
    const ticket = gate.begin();
    setLater(pageRequested(current, key));
    try {
      const page = await kcnaArchiveApi.search(queryFromString(key), offset);
      if (gate.isCurrent(ticket)) setLater((state) => pageReceived(state, key, page.items));
    } catch (error) {
      if (gate.isCurrent(ticket)) setLater((state) => pageFailed(state, key, failureKind(error)));
    }
  };

  /** The applied date range, with an ellipsis for an open end. */
  const rangeLabel = (): string => {
    const { from, to } = query();
    return `${from === null ? "…" : formatDate(from, locale())} – ${to === null ? "…" : formatDate(to, locale())}`;
  };

  const setQuery = (next: ArchiveQuery): void => {
    setParams(queryParams(next));
  };

  /** Where a result opens: beside the list, or on its own page when stacked. */
  const hrefFor = (id: string): string =>
    stacked() ? documentHref(id, query()) : searchHref(query(), { doc: id });

  useArchiveShortcuts((action) => {
    switch (action) {
      case "focus-search":
        return focusPrimaryInput();
      case "next":
      case "previous": {
        if (stacked()) return false;
        const list = items();
        const index = list.findIndex((item) => item.id === documentId());
        const target = list[action === "next" ? index + 1 : Math.max(0, index - 1)];
        if (target === undefined) return false;
        navigate(hrefFor(target.id), { replace: true, scroll: false });
        document
          .querySelector(`.kcna-row-item[data-id="${CSS.escape(target.id)}"]`)
          ?.scrollIntoView({ block: "nearest" });
        return true;
      }
      case "toggle-translation": {
        const translations = opened()?.translations ?? [];
        const target = translations.find((entry) => entry.language === "eng") ?? translations[0];
        if (target === undefined) return false;
        setParams({ with: besideLanguage() === undefined ? target.language : undefined });
        return true;
      }
    }
  });

  let readerPane: HTMLElement | undefined;
  // A new document starts at its top.
  createEffect(documentId, () => {
    readerPane?.scrollTo({ top: 0 });
  });

  const busy = createMemo(
    () => isPending(() => firstPage()) || isPending(() => opened()) || isPending(() => beside()),
  );

  const filters = (facets: Parameters<typeof FilterPanel>[0]["facets"]) => (
    <FilterPanel
      query={query()}
      facets={facets}
      onChange={setQuery}
      onReset={() => setQuery({ ...EMPTY_QUERY, text: query().text })}
    />
  );

  return (
    <>
      <SearchForm text={query().text} onSearch={(text) => setQuery({ ...query(), text })} />
      <div class={["kcna-progress", { "is-active": busy() }]} aria-hidden="true" />
      {/* Each pending read sits under its own boundary, so nothing waits on the
          slowest request: the filters appear at once and gain their counts. */}
      <Errored fallback={() => filters(undefined)}>
        <Loading fallback={filters(undefined)}>{filters(firstPage().facets)}</Loading>
      </Errored>
      <Errored fallback={() => <></>}>
        <Loading fallback={<ChartSkeleton />}>
          <Histogram
            buckets={histogram().buckets}
            from={query().from}
            to={query().to}
            onSelect={(range) => setQuery({ ...query(), from: range?.from ?? null, to: range?.to ?? null })}
          />
        </Loading>
      </Errored>
      <div class="kcna-panes">
        <section class="kcna-pane kcna-results" aria-label={t("kcna.results.label")} aria-busy={busy() ? "true" : "false"}>
          <Errored
            fallback={(error, reset) => (
              <Failure
                error={error()}
                onRetry={() => {
                  void refresh(firstPage);
                  reset();
                }}
              />
            )}
          >
            <Loading fallback={<ListSkeleton />}>
              <header class="kcna-pane-head">
                <h2>
                  {tx("kcna.results.count", { count: formatCount(firstPage().total, locale()) })}
                </h2>
                <Show when={query().from !== null || query().to !== null}>
                  <button
                    type="button"
                    class="kcna-toggle kcna-toggle--range"
                    title={t("kcna.filter.clear_range")}
                    aria-label={`${t("kcna.filter.clear_range")}: ${rangeLabel()}`}
                    onClick={() => setQuery({ ...query(), from: null, to: null })}
                  >
                    <span class="kcna-toggle-label" dir="ltr">
                      {rangeLabel()}
                    </span>
                    <span class="kcna-toggle-count" aria-hidden="true">
                      ×
                    </span>
                  </button>
                </Show>
              </header>
              <ResultList
                items={items()}
                total={Math.min(firstPage().total, RESULT_WINDOW)}
                selectedId={stacked() ? undefined : documentId()}
                loadingMore={laterPages().loading}
                failure={laterPages().failure}
                hrefFor={(item) => hrefFor(item.id)}
                keepScroll={!stacked()}
                onLoadMore={() => void loadMore()}
              />
            </Loading>
          </Errored>
        </section>
        <section
          ref={(element) => (readerPane = element)}
          class="kcna-pane kcna-reader-pane"
          aria-label={t("kcna.reader.label")}
        >
          <Errored
            fallback={(error, reset) => (
              <Failure
                error={error()}
                onRetry={() => {
                  void refresh(opened);
                  reset();
                }}
              />
            )}
          >
            <Loading fallback={<DocumentSkeleton />}>
              <Show when={opened()} fallback={<p class="kcna-empty">{t("kcna.reader.empty")}</p>}>
                {(current) => (
                  <Reader
                    document={current()}
                    beside={beside() ?? undefined}
                    besideLanguage={besideLanguage()}
                    hrefFor={(id) => searchHref(query(), { doc: id })}
                    keepScroll
                    onBeside={(language) => setParams({ with: language })}
                  />
                )}
              </Show>
            </Loading>
          </Errored>
        </section>
      </div>
    </>
  );
};

/** The route component: the section frame around this tab's page. */
const KcnaArchiveSearchPage: Component = () => (
  <KcnaArchiveLayout>
    <KcnaArchiveSearch />
  </KcnaArchiveLayout>
);

export default KcnaArchiveSearchPage;
