import type { ArchiveTextSegment } from "../../generated";

/** The text of a run of segments without the match marks. */
export const plainText = (segments: ReadonlyArray<ArchiveTextSegment>): string =>
  segments.map((segment) => segment.text).join("");
