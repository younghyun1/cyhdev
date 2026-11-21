// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveCategoryFacet } from "./archive-category-facet";
import type { ArchiveLanguageFacet } from "./archive-language-facet";
import type { ArchiveSourceFacet } from "./archive-source-facet";

export type ArchiveFacets = {
  readonly categories: ReadonlyArray<ArchiveCategoryFacet>;
  readonly languages: ReadonlyArray<ArchiveLanguageFacet>;
  readonly sources: ReadonlyArray<ArchiveSourceFacet>;
};
