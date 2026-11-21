// The archive query and its URL form. The URL is the single source of truth:
// every view is a link, and the back button restores the previous query.

import type { ArchiveLanguage, ArchiveSource } from "../../generated";
import {
  ARCHIVE_LANGUAGES,
  ARCHIVE_SOURCES,
  DEFAULT_SORT,
  SORT_ORDERS,
  type SortOrder,
} from "./constants";

export interface ArchiveQuery {
  /** Search text; empty browses. */
  readonly text: string;
  /** Empty selects every language. */
  readonly languages: ReadonlyArray<ArchiveLanguage>;
  /** Empty selects every source. */
  readonly sources: ReadonlyArray<ArchiveSource>;
  readonly category: string | null;
  /** Inclusive ISO dates. */
  readonly from: string | null;
  readonly to: string | null;
  readonly sort: SortOrder;
}

export const EMPTY_QUERY: ArchiveQuery = {
  text: "",
  languages: [],
  sources: [],
  category: null,
  from: null,
  to: null,
  sort: DEFAULT_SORT,
};

/** Longest search text the server accepts. */
export const MAX_TEXT_LENGTH = 200;

/** URL parameter names owned by the query. */
export const QUERY_KEYS = ["q", "lang", "source", "category", "from", "to", "sort"] as const;
type QueryKey = (typeof QUERY_KEYS)[number];
export type QueryParams = Readonly<Record<QueryKey, string | undefined>>;

type ParamSource = Readonly<Record<string, string | ReadonlyArray<string> | undefined>>;

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

/** The first value of a parameter that may repeat. */
export const firstParam = (value: string | ReadonlyArray<string> | undefined): string | undefined =>
  typeof value === "string" || value === undefined ? value : value[0];

const isDate = (value: string | undefined): value is string => {
  if (value === undefined || !ISO_DATE.test(value)) return false;
  const time = Date.parse(`${value}T00:00:00Z`);
  // Rejects impossible dates such as 2026-02-30, which `Date` would roll over.
  return !Number.isNaN(time) && new Date(time).toISOString().slice(0, 10) === value;
};

const members = <T extends string>(value: string | undefined, allowed: ReadonlyArray<T>): T[] => {
  if (value === undefined) return [];
  const requested = value.split(",").map((part) => part.trim());
  // Kept in the canonical order so equal selections give equal URLs.
  return allowed.filter((candidate) => requested.includes(candidate));
};

/** Reads a query from URL parameters, ignoring anything invalid. */
export const parseQuery = (params: ParamSource): ArchiveQuery => {
  const from = firstParam(params.from);
  const to = firstParam(params.to);
  const sort = firstParam(params.sort);
  const category = firstParam(params.category)?.trim();
  const validFrom = isDate(from) ? from : null;
  const validTo = isDate(to) ? to : null;
  const ordered = validFrom === null || validTo === null || validFrom <= validTo;
  return {
    text: (firstParam(params.q) ?? "").trim().slice(0, MAX_TEXT_LENGTH),
    languages: members(firstParam(params.lang), ARCHIVE_LANGUAGES),
    sources: members(firstParam(params.source), ARCHIVE_SOURCES),
    category: category === undefined || category === "" ? null : category,
    from: ordered ? validFrom : null,
    to: ordered ? validTo : null,
    sort: SORT_ORDERS.find((order) => order === sort) ?? DEFAULT_SORT,
  };
};

/** URL parameters for a query; defaults are `undefined` so they are omitted. */
export const queryParams = (query: ArchiveQuery): QueryParams => ({
  q: query.text === "" ? undefined : query.text,
  lang: query.languages.length === 0 ? undefined : query.languages.join(","),
  source: query.sources.length === 0 ? undefined : query.sources.join(","),
  category: query.category ?? undefined,
  from: query.from ?? undefined,
  to: query.to ?? undefined,
  sort: query.sort === DEFAULT_SORT ? undefined : query.sort,
});

/** A stable string form, used as a change key and as a link query string. */
export const queryString = (
  query: ArchiveQuery,
  extra: Readonly<Record<string, string | number | undefined>> = {},
): string => {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(queryParams(query))) {
    if (value !== undefined) search.set(key, value);
  }
  for (const [key, value] of Object.entries(extra)) {
    if (value !== undefined) search.set(key, String(value));
  }
  return search.toString();
};

/** Reads a query back from the string form `queryString` produced. */
export const queryFromString = (key: string): ArchiveQuery =>
  parseQuery(Object.fromEntries(new URLSearchParams(key)));

/** The query without its date range and sort order, as the histogram uses it. */
export const withoutRange = (query: ArchiveQuery): ArchiveQuery => ({
  ...query,
  from: null,
  to: null,
  sort: DEFAULT_SORT,
});

export const toggled = <T>(values: ReadonlyArray<T>, value: T): T[] =>
  values.includes(value) ? values.filter((entry) => entry !== value) : [...values, value];

export const isFiltered = (query: ArchiveQuery): boolean =>
  queryString(query) !== queryString(EMPTY_QUERY);
