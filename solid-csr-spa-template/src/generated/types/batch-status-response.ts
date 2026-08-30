// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { BatchItemStatus } from "./batch-item-status";

export type BatchStatusResponse = {
  readonly batch_id: string;
  readonly completed: number;
  readonly created_at: string;
  readonly done: boolean;
  readonly failed: number;
  readonly items: ReadonlyArray<BatchItemStatus>;
  readonly pending: number;
  readonly total: number;
};
