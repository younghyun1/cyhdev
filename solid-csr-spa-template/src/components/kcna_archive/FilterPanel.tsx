import { createSignal, For, Show, type Component } from "solid-js";

import type {
  ArchiveFacets,
  ArchiveLanguage,
  ArchiveSource,
} from "../../services/contracts/kcna_archive";
import { exportHref } from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import {
  ARCHIVE_LANGUAGES,
  ARCHIVE_SOURCES,
  DATE_MAX,
  DATE_MIN,
  SORT_ORDERS,
  languageKey,
  sortKey,
  sourceKey,
  type SortOrder,
} from "../../utils/kcna_archive/constants";
import { parseDateJump } from "../../utils/kcna_archive/dates";
import { formatCount } from "../../utils/kcna_archive/format";
import { isFiltered, toggled, type ArchiveQuery } from "../../utils/kcna_archive/query";

interface FilterPanelProps {
  readonly query: ArchiveQuery;
  /** Counts under the current filters; absent while they load or after a failure. */
  readonly facets: ArchiveFacets | undefined;
  readonly onChange: (query: ArchiveQuery) => void;
  readonly onReset: () => void;
}

const isSort = (value: string): value is SortOrder => SORT_ORDERS.some((order) => order === value);

/** A pressed-or-not filter button showing how many results it holds. */
const FilterToggle: Component<{
  readonly label: string;
  readonly count: number | undefined;
  readonly pressed: boolean;
  readonly onToggle: () => void;
}> = (props) => {
  const shown = (): string => (props.count === undefined ? "" : formatCount(props.count, locale()));
  return (
    <button
      type="button"
      class="kcna-toggle"
      aria-pressed={props.pressed ? "true" : "false"}
      title={shown() === "" ? props.label : `${props.label} (${shown()})`}
      onClick={() => props.onToggle()}
    >
      <span class="kcna-toggle-label">{props.label}</span>
      <Show when={shown() !== ""}>
        <span class="kcna-toggle-count">{shown()}</span>
      </Show>
    </button>
  );
};

const countOf = <T,>(
  values: ReadonlyArray<{ readonly value: T; readonly count: number }> | undefined,
  value: T,
): number | undefined => values?.find((entry) => entry.value === value)?.count;

/** Language and source toggles, category, date range, order, date jump, and export. */
const FilterPanel: Component<FilterPanelProps> = (props) => {
  const [jump, setJump] = createSignal("");
  const [jumpInvalid, setJumpInvalid] = createSignal(false);

  const setDate = (key: "from" | "to", value: string): void => {
    props.onChange({ ...props.query, [key]: value === "" ? null : value });
  };

  const toggle = <T extends ArchiveLanguage | ArchiveSource>(
    key: "languages" | "sources",
    current: ReadonlyArray<T>,
    value: T,
  ): void => {
    props.onChange({ ...props.query, [key]: toggled(current, value) });
  };

  const submitJump = (event: SubmitEvent): void => {
    event.preventDefault();
    const range = parseDateJump(jump());
    setJumpInvalid(range === undefined);
    if (range === undefined) return;
    props.onChange({ ...props.query, from: range.from, to: range.to });
    setJump("");
  };

  return (
    <section class="kcna-filters" aria-label={t("kcna.results.label")}>
      <fieldset class="kcna-filter-group">
        <legend class="kcna-filter-legend">{t("kcna.filter.language")}</legend>
        <div class="kcna-toggle-grid kcna-toggle-grid--languages">
          <For each={ARCHIVE_LANGUAGES}>
            {(language) => (
              <FilterToggle
                label={t(languageKey(language))}
                count={countOf(props.facets?.languages, language)}
                pressed={props.query.languages.includes(language)}
                onToggle={() => toggle("languages", props.query.languages, language)}
              />
            )}
          </For>
        </div>
      </fieldset>
      <fieldset class="kcna-filter-group">
        <legend class="kcna-filter-legend">{t("kcna.filter.source")}</legend>
        <div class="kcna-toggle-grid kcna-toggle-grid--sources">
          <For each={ARCHIVE_SOURCES}>
            {(source) => (
              <FilterToggle
                label={t(sourceKey(source))}
                count={countOf(props.facets?.sources, source)}
                pressed={props.query.sources.includes(source)}
                onToggle={() => toggle("sources", props.query.sources, source)}
              />
            )}
          </For>
        </div>
      </fieldset>
      <div class="kcna-fields">
        <Show when={(props.facets?.categories.length ?? 0) > 0 || props.query.category !== null}>
          <label class="kcna-field">
            <span class="kcna-field-label" title={t("kcna.filter.category")}>{t("kcna.filter.category")}</span>
            <select
              class="kcna-select"
              value={props.query.category ?? ""}
              onChange={(event) => {
                const value = event.currentTarget.value;
                props.onChange({ ...props.query, category: value === "" ? null : value });
              }}
            >
              <option value="">{t("kcna.filter.all_categories")}</option>
              <Show when={props.query.category !== null && props.facets?.categories.every((entry) => entry.value !== props.query.category)}>
                <option value={props.query.category ?? ""}>{props.query.category}</option>
              </Show>
              <For each={props.facets?.categories ?? []}>
                {(category) => (
                  <option value={category.value}>
                    {category.value} ({formatCount(category.count, locale())})
                  </option>
                )}
              </For>
            </select>
          </label>
        </Show>
        <label class="kcna-field">
          <span class="kcna-field-label" title={t("kcna.filter.from")}>{t("kcna.filter.from")}</span>
          <input
            class="kcna-input"
            type="date"
            min={DATE_MIN}
            max={DATE_MAX}
            value={props.query.from ?? ""}
            onChange={(event) => setDate("from", event.currentTarget.value)}
          />
        </label>
        <label class="kcna-field">
          <span class="kcna-field-label" title={t("kcna.filter.to")}>{t("kcna.filter.to")}</span>
          <input
            class="kcna-input"
            type="date"
            min={DATE_MIN}
            max={DATE_MAX}
            value={props.query.to ?? ""}
            onChange={(event) => setDate("to", event.currentTarget.value)}
          />
        </label>
        <label class="kcna-field">
          <span class="kcna-field-label" title={t("kcna.sort.label")}>{t("kcna.sort.label")}</span>
          <select
            class="kcna-select"
            value={props.query.sort}
            onChange={(event) => {
              const value = event.currentTarget.value;
              if (isSort(value)) props.onChange({ ...props.query, sort: value });
            }}
          >
            <For each={SORT_ORDERS}>{(order) => <option value={order}>{t(sortKey(order))}</option>}</For>
          </select>
        </label>
      </div>
      <div class="kcna-filter-footer">
        <form class="kcna-jump" onSubmit={submitJump}>
          <label class="kcna-field kcna-field--grow">
            <span class="kcna-field-label" title={t("kcna.jump.label")}>{t("kcna.jump.label")}</span>
            <span class="kcna-jump-row">
              <input
                class="kcna-input"
                type="text"
                inputmode="numeric"
                dir="ltr"
                autocomplete="off"
                placeholder={t("kcna.jump.placeholder")}
                aria-invalid={jumpInvalid() ? "true" : undefined}
                value={jump()}
                onInput={(event) => {
                  setJump(event.currentTarget.value);
                  setJumpInvalid(false);
                }}
              />
              <button type="submit" class="kcna-button">
                {t("kcna.jump.submit")}
              </button>
            </span>
          </label>
        </form>
        <div class="kcna-actions" role="group" aria-label={t("kcna.results.export")}>
          <span class="kcna-actions-label">{t("kcna.results.export")}</span>
          <a class="kcna-button kcna-button--link" href={exportHref(props.query, "csv")} download>
            CSV
          </a>
          <a class="kcna-button kcna-button--link" href={exportHref(props.query, "jsonl")} download>
            JSONL
          </a>
          <Show when={isFiltered({ ...props.query, text: "" })}>
            <button type="button" class="kcna-button" onClick={() => props.onReset()}>
              {t("kcna.filter.reset")}
            </button>
          </Show>
        </div>
      </div>
    </section>
  );
};

export default FilterPanel;
