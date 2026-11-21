import {
  createEffect,
  createMemo,
  createSignal,
  Errored,
  For,
  Loading,
  refresh,
  Show,
  type Component,
} from "solid-js";
import { useSearchParams } from "@solidjs/router";

import { ChartSkeleton, Failure } from "../../components/kcna_archive/Notice";
import TrendChart from "../../components/kcna_archive/TrendChart";
import { focusPrimaryInput, useArchiveShortcuts } from "../../components/kcna_archive/useArchiveShortcuts";
import { kcnaArchiveApi, type ArchiveBucket, type ArchiveLanguage } from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { ARCHIVE_LANGUAGES, languageKey } from "../../utils/kcna_archive/constants";
import { formatCount, formatMonth, formatPercent, formatYear } from "../../utils/kcna_archive/format";
import { EMPTY_QUERY, firstParam } from "../../utils/kcna_archive/query";
import {
  parseTerms,
  share,
  trendsCsv,
  type TrendMode,
} from "../../utils/kcna_archive/trends";
import KcnaArchiveLayout from "./Layout";

/** The table of every period; the CSV carries the same numbers. */
const KcnaArchiveTrends: Component = () => {
  const [params, setParams] = useSearchParams();

  const termsKey = createMemo(() => parseTerms(firstParam(params.terms) ?? "").join(","));
  const language = createMemo((): ArchiveLanguage | undefined =>
    ARCHIVE_LANGUAGES.find((candidate) => candidate === firstParam(params.lang)),
  );
  const bucket = createMemo((): ArchiveBucket => (firstParam(params.bucket) === "month" ? "month" : "year"));
  const mode = createMemo((): TrendMode => (firstParam(params.mode) === "count" ? "count" : "share"));
  // What is typed; it follows the URL when the URL changes.
  const [draft, setDraft] = createSignal("");
  createEffect(
    () => firstParam(params.terms) ?? "",
    (terms) => {
      setDraft(terms);
    },
  );

  const trends = createMemo(async () => {
    const terms = termsKey() === "" ? [] : termsKey().split(",");
    const selected = language();
    const size = bucket();
    if (terms.length === 0) return null;
    return kcnaArchiveApi.trends(
      terms,
      { ...EMPTY_QUERY, languages: selected === undefined ? [] : [selected] },
      size,
    );
  });

  useArchiveShortcuts((action) => (action === "focus-search" ? focusPrimaryInput() : false));

  const label = (start: string, size: ArchiveBucket): string =>
    size === "year" ? formatYear(start, locale()) : formatMonth(start, locale());

  const download = (csv: string): void => {
    const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "kcna-trends.csv";
    link.click();
    URL.revokeObjectURL(url);
  };

  const segment = (name: string, active: boolean, onClick: () => void) => (
    <button
      type="button"
      class="kcna-toggle"
      aria-pressed={active ? "true" : "false"}
      title={name}
      onClick={onClick}
    >
      <span class="kcna-toggle-label">{name}</span>
    </button>
  );

  return (
    <section class="kcna-trends" aria-label={t("kcna.trends.title")}>
      <h2 class="section-title kcna-section-title">{t("kcna.trends.title")}</h2>
      <form
        class="kcna-trends-form"
        onSubmit={(event) => {
          event.preventDefault();
          setParams({ terms: parseTerms(draft()).join(",") || undefined });
        }}
      >
        <div class="kcna-search">
          <label class="sr-only" for="kcna-trends-input">
            {t("kcna.trends.placeholder")}
          </label>
          <input
            id="kcna-trends-input"
            class="kcna-input"
            type="text"
            dir="auto"
            data-kcna-primary-input
            autocomplete="off"
            placeholder={t("kcna.trends.placeholder")}
            value={draft()}
            onInput={(event) => setDraft(event.currentTarget.value)}
          />
          <button type="submit" class="kcna-button kcna-button--primary">
            {t("kcna.trends.compare")}
          </button>
        </div>
        <div class="kcna-fields kcna-fields--trends">
          <label class="kcna-field">
            <span class="kcna-field-label" title={t("kcna.filter.language")}>{t("kcna.filter.language")}</span>
            <select
              class="kcna-select"
              value={language() ?? ""}
              onChange={(event) => setParams({ lang: event.currentTarget.value || undefined })}
            >
              <option value="">{t("kcna.filter.language")}</option>
              <For each={ARCHIVE_LANGUAGES}>
                {(candidate) => <option value={candidate}>{t(languageKey(candidate))}</option>}
              </For>
            </select>
          </label>
          <div class="kcna-segments" role="group" aria-label={`${t("kcna.trends.year")} / ${t("kcna.trends.month")}`}>
            {segment(t("kcna.trends.year"), bucket() === "year", () => setParams({ bucket: undefined }))}
            {segment(t("kcna.trends.month"), bucket() === "month", () => setParams({ bucket: "month" }))}
          </div>
          <div class="kcna-segments" role="group" aria-label={`${t("kcna.trends.share")} / ${t("kcna.trends.count")}`}>
            {segment(t("kcna.trends.share"), mode() === "share", () => setParams({ mode: undefined }))}
            {segment(t("kcna.trends.count"), mode() === "count", () => setParams({ mode: "count" }))}
          </div>
        </div>
      </form>
      <Errored
        fallback={(error, reset) => (
          <Failure
            error={error()}
            onRetry={() => {
              void refresh(trends);
              reset();
            }}
          />
        )}
      >
        <Loading fallback={<ChartSkeleton />}>
          <Show when={trends()} fallback={<p class="kcna-empty">{t("kcna.trends.empty")}</p>}>
            {(data) => (
              <>
                <TrendChart data={data()} mode={mode()} />
                <p class="kcna-table-actions">
                  <button type="button" class="kcna-button" onClick={() => download(trendsCsv(data()))}>
                    {t("kcna.trends.download")}
                  </button>
                </p>
                <div
                  class="kcna-table-scroll"
                  role="region"
                  tabindex="0"
                  aria-label={t("kcna.trends.title")}
                  style={{ "--kcna-terms": String(data().series.length) }}
                >
                  <table class="kcna-table">
                    <thead>
                      <tr>
                        <th scope="col" title={t("kcna.trends.period")}>
                          {t("kcna.trends.period")}
                        </th>
                        <th scope="col" title={t("kcna.trends.total")}>
                          {t("kcna.trends.total")}
                        </th>
                        <For each={data().series}>
                          {(series) => (
                            <th scope="col" dir="auto" title={series.term}>
                              {series.term}
                            </th>
                          )}
                        </For>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={data().starts}>
                        {(start, row) => (
                          <tr>
                            <th scope="row">{label(start, data().bucket)}</th>
                            <td>{formatCount(data().totals[row()] ?? 0, locale())}</td>
                            <For each={data().series}>
                              {(series) => (
                                <td>
                                  {formatCount(series.counts[row()] ?? 0, locale())}
                                  <small>
                                    {formatPercent(
                                      share(series.counts[row()] ?? 0, data().totals[row()] ?? 0),
                                      locale(),
                                    )}
                                  </small>
                                </td>
                              )}
                            </For>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </div>
              </>
            )}
          </Show>
        </Loading>
      </Errored>
    </section>
  );
};

/** The route component: the section frame around this tab's page. */
const KcnaArchiveTrendsPage: Component = () => (
  <KcnaArchiveLayout>
    <KcnaArchiveTrends />
  </KcnaArchiveLayout>
);

export default KcnaArchiveTrendsPage;
