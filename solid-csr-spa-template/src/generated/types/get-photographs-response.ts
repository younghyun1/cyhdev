// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { PaginationMeta } from "./pagination-meta";
import type { PhotographItem } from "./photograph-item";

export type GetPhotographsResponse = {
  readonly items: ReadonlyArray<PhotographItem>;
  readonly pagination: PaginationMeta;
};
