const UNITS: ReadonlyArray<readonly [Intl.RelativeTimeFormatUnit, number]> = [
  ["year", 31_536_000],
  ["month", 2_592_000],
  ["week", 604_800],
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];

/** One formatter per UI locale; the locale list is fixed, so this stays bounded. */
const formatters = new Map<string, Intl.RelativeTimeFormat>();

function formatterFor(locale: string): Intl.RelativeTimeFormat {
  const cached = formatters.get(locale);
  if (cached) return cached;
  const formatter = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  formatters.set(locale, formatter);
  return formatter;
}

/** "3 hours ago" in `locale`; an unparseable timestamp yields an empty string. */
export function formatRelativeTime(
  iso: string,
  locale: string,
  now: number = Date.now(),
): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const seconds = Math.round((then - now) / 1000);
  const magnitude = Math.abs(seconds);
  for (const [unit, size] of UNITS) {
    if (magnitude >= size) {
      return formatterFor(locale).format(Math.trunc(seconds / size), unit);
    }
  }
  return formatterFor(locale).format(0, "second");
}
