// Geometry and selection logic for the time histogram, kept free of the DOM so
// it can be tested.

import type { ArchiveHistogramBucket } from "../../generated";
import { monthEnd, monthIndex, monthStart } from "./dates";

export interface MonthBar {
  /** Month index (see `monthIndex`). */
  readonly month: number;
  readonly start: string;
  readonly count: number;
}

/** One bar per month from the first to the last bucket, gaps filled with zero. */
export const monthBars = (buckets: ReadonlyArray<ArchiveHistogramBucket>): MonthBar[] => {
  const first = buckets[0];
  const last = buckets[buckets.length - 1];
  if (first === undefined || last === undefined) return [];
  const counts = new Map(buckets.map((bucket) => [monthIndex(bucket.start), bucket.count]));
  const bars: MonthBar[] = [];
  for (let month = monthIndex(first.start); month <= monthIndex(last.start); month += 1) {
    bars.push({ month, start: monthStart(month), count: counts.get(month) ?? 0 });
  }
  return bars;
};

/**
 * Bar height as a share of the chart, on a square-root scale: months range
 * from a handful of documents to thousands, and a linear scale would flatten
 * the sparse years to nothing.
 */
export const barHeight = (count: number, maximum: number): number =>
  maximum <= 0 || count <= 0 ? 0 : Math.max(0.02, Math.sqrt(count / maximum));

/** The bar under a horizontal position given as a share of the chart width. */
export const barAt = (bars: ReadonlyArray<MonthBar>, share: number): MonthBar | undefined => {
  if (bars.length === 0) return undefined;
  const index = Math.min(bars.length - 1, Math.max(0, Math.floor(share * bars.length)));
  return bars[index];
};

/** The inclusive date range covered by two bars, in either order. */
export const rangeOf = (first: MonthBar, second: MonthBar): { from: string; to: string } => {
  const low = Math.min(first.month, second.month);
  const high = Math.max(first.month, second.month);
  return { from: monthStart(low), to: monthEnd(high) };
};

/** Whether a bar lies inside an inclusive date range; open ends select all. */
export const inRange = (bar: MonthBar, from: string | null, to: string | null): boolean =>
  (from === null || monthEnd(bar.month) >= from) && (to === null || bar.start <= to);

/** Year labels for the axis: the January bars, thinned to fit `slots` labels. */
export const yearTicks = (bars: ReadonlyArray<MonthBar>, slots: number): MonthBar[] => {
  const januaries = bars.filter((bar) => bar.month % 12 === 0);
  const step = Math.max(1, Math.ceil(januaries.length / Math.max(1, slots)));
  return januaries.filter((_, index) => index % step === 0);
};
