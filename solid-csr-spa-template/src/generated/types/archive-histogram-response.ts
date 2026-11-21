// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveBucket } from "./archive-bucket";
import type { ArchiveHistogramBucket } from "./archive-histogram-bucket";

export type ArchiveHistogramResponse = {
  readonly bucket: ArchiveBucket;
  readonly buckets: ReadonlyArray<ArchiveHistogramBucket>;
};
