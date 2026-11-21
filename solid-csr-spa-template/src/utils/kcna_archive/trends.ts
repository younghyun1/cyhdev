// Pure helpers for the trends view.

import type { ArchiveBucket, ArchiveTrendsResponse } from "../../generated";
import { MAX_TERMS } from "./constants";

export type TrendMode = "share" | "count";

/** Splits user input into at most four distinct, non-empty terms. */
export const parseTerms = (input: string): string[] => {
  const terms: string[] = [];
  for (const part of input.split(/[,，、\n]/)) {
    const term = part.trim().slice(0, 100);
    if (term !== "" && !terms.includes(term)) terms.push(term);
    if (terms.length === MAX_TERMS) break;
  }
  return terms;
};

/** Share of a bucket's documents that match, in percent; zero for an empty bucket. */
export const share = (count: number, total: number): number =>
  total <= 0 ? 0 : (count / total) * 100;

/** The values plotted for each series: shares or raw counts. */
export const plotted = (trends: ArchiveTrendsResponse, mode: TrendMode): number[][] =>
  trends.series.map((series) =>
    series.counts.map((count, index) =>
      mode === "count" ? count : share(count, trends.totals[index] ?? 0),
    ),
  );

/** The bucket labels' source dates are `YYYY-MM-DD`; year buckets show only the year. */
export const periodLabel = (start: string, bucket: ArchiveBucket): string =>
  bucket === "year" ? start.slice(0, 4) : start.slice(0, 7);

/** SVG path through `values` in a `width` by `height` box, scaled to `maximum`. */
export const linePath = (
  values: ReadonlyArray<number>,
  maximum: number,
  width: number,
  height: number,
): string => {
  if (values.length === 0) return "";
  const step = values.length === 1 ? 0 : width / (values.length - 1);
  return values
    .map((value, index) => {
      const x = index * step;
      const y = maximum <= 0 ? height : height - (value / maximum) * height;
      return `${index === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`;
    })
    .join(" ");
};

/** The table as CSV: one row per period, count and share per term. */
export const trendsCsv = (trends: ArchiveTrendsResponse): string => {
  const quote = (value: string): string =>
    /[",\n\r]/.test(value) ? `"${value.replaceAll('"', '""')}"` : value;
  const header = [
    "period",
    "documents",
    ...trends.series.flatMap((series) => [`${series.term} (count)`, `${series.term} (share %)`]),
  ];
  const rows = trends.starts.map((start, index) => {
    const total = trends.totals[index] ?? 0;
    return [
      periodLabel(start, trends.bucket),
      String(total),
      ...trends.series.flatMap((series) => {
        const count = series.counts[index] ?? 0;
        return [String(count), share(count, total).toFixed(3)];
      }),
    ];
  });
  return [header, ...rows].map((row) => row.map(quote).join(",")).join("\r\n");
};
