// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveBucket } from "./archive-bucket";
import type { ArchiveTrendSeries } from "./archive-trend-series";

export type ArchiveTrendsResponse = {
  readonly bucket: ArchiveBucket;
  readonly series: ReadonlyArray<ArchiveTrendSeries>;
  readonly starts: ReadonlyArray<string>;
  readonly totals: ReadonlyArray<number>;
};
