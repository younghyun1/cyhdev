// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveLanguage } from "./archive-language";
import type { ArchiveSource } from "./archive-source";
import type { ArchiveTextSegment } from "./archive-text-segment";

export type ArchiveSearchItem = {
  readonly category?: string | null;
  readonly hasFullText: boolean;
  readonly id: string;
  readonly language: ArchiveLanguage;
  readonly publicationDate: string;
  readonly snippet: ReadonlyArray<ArchiveTextSegment>;
  readonly sources: ReadonlyArray<ArchiveSource>;
  readonly title: ReadonlyArray<ArchiveTextSegment>;
};
