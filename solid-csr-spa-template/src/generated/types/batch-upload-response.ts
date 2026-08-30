// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { BatchUploadItem } from "./batch-upload-item";

export type BatchUploadResponse = {
  readonly batch_id: string;
  readonly items: ReadonlyArray<BatchUploadItem>;
  readonly total: number;
};
