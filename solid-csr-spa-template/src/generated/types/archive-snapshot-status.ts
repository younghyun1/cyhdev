// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveLanguageFacet } from "./archive-language-facet";
import type { ArchiveSourceFacet } from "./archive-source-facet";

export type ArchiveSnapshotStatus = {
  readonly copies: number;
  readonly documents: number;
  readonly indexed: number;
  readonly languages: ReadonlyArray<ArchiveLanguageFacet>;
  readonly lastRetrievedAt?: string | null;
  readonly memoryBytes?: number | null;
  readonly newestDate?: string | null;
  readonly oldestDate?: string | null;
  readonly snapshotBuiltAt: string;
  readonly sources: ReadonlyArray<ArchiveSourceFacet>;
  readonly translationGroups: number;
};
