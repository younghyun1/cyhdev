// Number and date formatting in the selected interface language. Formatters
// are cached per locale tag; the tag list is fixed, so the caches stay small.

const numbers = new Map<string, Intl.NumberFormat>();
const percents = new Map<string, Intl.NumberFormat>();
const megabytes = new Map<string, Intl.NumberFormat>();
const gigabytes = new Map<string, Intl.NumberFormat>();
const dates = new Map<string, Intl.DateTimeFormat>();
const timestamps = new Map<string, Intl.DateTimeFormat>();
const months = new Map<string, Intl.DateTimeFormat>();
const years = new Map<string, Intl.DateTimeFormat>();

const FALLBACK = "en-US";

/** A formatter for `tag`, or for English when the runtime rejects the tag. */
function formatterFor<F>(
  cache: Map<string, F>,
  tag: string,
  create: (tag: string) => F,
): F {
  const cached = cache.get(tag);
  if (cached !== undefined) return cached;
  let formatter: F;
  try {
    formatter = create(tag);
  } catch {
    formatter = create(FALLBACK);
  }
  cache.set(tag, formatter);
  return formatter;
}

export const formatCount = (value: number, tag: string): string =>
  formatterFor(numbers, tag, (locale) => new Intl.NumberFormat(locale)).format(value);

/** Parses `YYYY-MM-DD` as midnight UTC, so the reader's time zone never moves the day. */
const utcDay = (date: string): Date => new Date(`${date.slice(0, 10)}T00:00:00Z`);

const validDate = (value: Date): boolean => !Number.isNaN(value.getTime());

/** A calendar date such as `Sep 2, 2026`; unparseable input is returned unchanged. */
export const formatDate = (date: string, tag: string): string => {
  const value = utcDay(date);
  if (!validDate(value)) return date;
  return formatterFor(
    dates,
    tag,
    (locale) => new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeZone: "UTC" }),
  ).format(value);
};

/** An RFC 3339 instant with its time zone, always in UTC. */
export const formatTimestamp = (instant: string, tag: string): string => {
  const value = new Date(instant);
  if (!validDate(value)) return instant;
  return formatterFor(
    timestamps,
    tag,
    (locale) =>
      new Intl.DateTimeFormat(locale, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
        timeZone: "UTC",
        timeZoneName: "short",
      }),
  ).format(value);
};

/** A month such as `Sep 2026`, for chart tooltips and axes. */
export const formatMonth = (date: string, tag: string): string => {
  const value = utcDay(date);
  if (!validDate(value)) return date.slice(0, 7);
  return formatterFor(
    months,
    tag,
    (locale) => new Intl.DateTimeFormat(locale, { year: "numeric", month: "short", timeZone: "UTC" }),
  ).format(value);
};

export const formatYear = (date: string, tag: string): string => {
  const value = utcDay(date);
  if (!validDate(value)) return date.slice(0, 4);
  return formatterFor(
    years,
    tag,
    (locale) => new Intl.DateTimeFormat(locale, { year: "numeric", timeZone: "UTC" }),
  ).format(value);
};

const MEGABYTE = 1024 * 1024;
const GIGABYTE = 1024 * MEGABYTE;

const unitFormat = (unit: "megabyte" | "gigabyte") => (locale: string): Intl.NumberFormat =>
  new Intl.NumberFormat(locale, { style: "unit", unit, unitDisplay: "short", maximumFractionDigits: 1 });

/** A byte count in megabytes or gigabytes, as the locale writes them. */
export const formatBytes = (bytes: number, tag: string): string =>
  bytes >= GIGABYTE
    ? formatterFor(gigabytes, tag, unitFormat("gigabyte")).format(bytes / GIGABYTE)
    : formatterFor(megabytes, tag, unitFormat("megabyte")).format(bytes / MEGABYTE);

/** A percentage such as `12.50%`; the argument is already in percent, not a fraction. */
export const formatPercent = (percent: number, tag: string): string =>
  formatterFor(
    percents,
    tag,
    (locale) =>
      new Intl.NumberFormat(locale, {
        style: "percent",
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      }),
  ).format(percent / 100);
