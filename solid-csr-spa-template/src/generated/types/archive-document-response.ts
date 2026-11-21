// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveCopyLink } from "./archive-copy-link";
import type { ArchiveLanguage } from "./archive-language";
import type { ArchivePrecision } from "./archive-precision";
import type { ArchiveSource } from "./archive-source";
import type { ArchiveTextSegment } from "./archive-text-segment";
import type { ArchiveTranslationLink } from "./archive-translation-link";

export type ArchiveDocumentResponse = {
  readonly canonicalUrl: string;
  readonly category?: string | null;
  readonly copies: ReadonlyArray<ArchiveCopyLink>;
  readonly id: string;
  readonly language: ArchiveLanguage;
  readonly metadata: unknown;
  readonly paragraphs: ReadonlyArray<ReadonlyArray<ArchiveTextSegment>>;
  readonly publicationDate: string;
  readonly publicationPrecision: ArchivePrecision;
  readonly source: ArchiveSource;
  readonly sourceRecordId?: string | null;
  readonly sources: ReadonlyArray<ArchiveSource>;
  readonly title: ReadonlyArray<ArchiveTextSegment>;
  readonly translations: ReadonlyArray<ArchiveTranslationLink>;
};
