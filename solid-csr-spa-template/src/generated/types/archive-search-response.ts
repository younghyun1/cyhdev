// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveFacets } from "./archive-facets";
import type { ArchiveSearchItem } from "./archive-search-item";

export type ArchiveSearchResponse = {
  readonly facets: ArchiveFacets;
  readonly items: ReadonlyArray<ArchiveSearchItem>;
  readonly limit: number;
  readonly offset: number;
  readonly total: number;
};
