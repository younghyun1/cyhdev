// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ProcessingStatus } from "./processing-status";

export type BatchItemStatus = {
  readonly created_at: string;
  readonly file_name?: string | null;
  readonly item_id: string;
  readonly original_size_bytes: number;
  readonly status: ProcessingStatus;
  readonly updated_at: string;
};
