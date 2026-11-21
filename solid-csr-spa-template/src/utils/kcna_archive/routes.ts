import { ARCHIVE_LANGUAGES } from "./constants";
import type { ArchiveLanguage } from "../../generated";
import { firstParam, queryString, type ArchiveQuery } from "./query";

/** The archive section's root; every archive route lives below it. */
export const ARCHIVE_PATH = "/projects/kcna-archive";
export const ARCHIVE_TRENDS_PATH = `${ARCHIVE_PATH}/trends`;
export const ARCHIVE_STATUS_PATH = `${ARCHIVE_PATH}/status`;

/** The reader page of one document, carrying the search it was opened from. */
export const documentHref = (
  id: string,
  query: ArchiveQuery,
  extra: Readonly<Record<string, string | undefined>> = {},
): string => {
  const carried = queryString(query, extra);
  const path = `${ARCHIVE_PATH}/documents/${encodeURIComponent(id)}`;
  return carried === "" ? path : `${path}?${carried}`;
};

/** The search page with a document open beside the results. */
export const searchHref = (
  query: ArchiveQuery,
  extra: Readonly<Record<string, string | undefined>> = {},
): string => {
  const carried = queryString(query, extra);
  return carried === "" ? ARCHIVE_PATH : `${ARCHIVE_PATH}?${carried}`;
};

/** The `with` parameter: the language edition shown beside the document. */
export const besideLanguageOf = (
  value: string | ReadonlyArray<string> | undefined,
): ArchiveLanguage | undefined => {
  const requested = firstParam(value);
  return ARCHIVE_LANGUAGES.find((language) => language === requested);
};

/** Which tab a pathname belongs to; the reader counts as part of search. */
export type ArchiveSection = "search" | "trends" | "status";

export const sectionOf = (pathname: string): ArchiveSection | undefined => {
  const trimmed = pathname.length > 1 ? pathname.replace(/\/+$/, "") : pathname;
  if (trimmed === ARCHIVE_TRENDS_PATH) return "trends";
  if (trimmed === ARCHIVE_STATUS_PATH) return "status";
  if (trimmed === ARCHIVE_PATH || trimmed.startsWith(`${ARCHIVE_PATH}/documents/`)) return "search";
  return undefined;
};
