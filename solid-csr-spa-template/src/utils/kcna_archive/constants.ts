import type { ArchiveLanguage, ArchiveSource, ArchivePrecision } from "../../generated";
import type { UiTextKey } from "../../i18n/keys";

/** Languages the archive holds, in display order. */
export const ARCHIVE_LANGUAGES = [
  "kor",
  "eng",
  "zho",
  "rus",
  "spa",
  "jpn",
] as const satisfies ReadonlyArray<ArchiveLanguage>;

/** Publishers of article copies, in display order. */
export const ARCHIVE_SOURCES = [
  "kcna_kp",
  "kcna_co_jp",
  "stiles_archive",
] as const satisfies ReadonlyArray<ArchiveSource>;

/** Resolves to `never` only when `List` names every member of `Union`. */
type AssertCovered<Missing extends never> = Missing;

/** Compile-time proof that the lists above cover every member of the generated unions. */
export type ArchiveListsCovered = [
  AssertCovered<Exclude<ArchiveLanguage, (typeof ARCHIVE_LANGUAGES)[number]>>,
  AssertCovered<Exclude<ArchiveSource, (typeof ARCHIVE_SOURCES)[number]>>,
];

export const SORT_ORDERS = ["newest", "oldest", "relevance"] as const;
export type SortOrder = (typeof SORT_ORDERS)[number];

/** A news archive is read by date, so results are newest first unless asked otherwise. */
export const DEFAULT_SORT: SortOrder = "newest";

/** Results requested per page. The server accepts 1 to 100. */
export const PAGE_SIZE = 50;

/** The server refuses to page past this many results. */
export const RESULT_WINDOW = 10_000;

/** Most terms the trends view compares. */
export const MAX_TERMS = 4;

/** Earliest and latest year the date controls accept. */
export const DATE_MIN = "1990-01-01";
export const DATE_MAX = "2100-12-31";

export const languageKey = (language: ArchiveLanguage): UiTextKey =>
  `kcna.language.${language}`;

export const sourceKey = (source: ArchiveSource): UiTextKey => `kcna.source.${source}`;

export const sortKey = (order: SortOrder): UiTextKey => `kcna.sort.${order}`;

export const precisionKey = (precision: ArchivePrecision): UiTextKey =>
  `kcna.precision.${precision}`;
