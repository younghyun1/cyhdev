// Pairs the paragraphs of two language editions for side-by-side reading.

import type { ArchiveTextSegment } from "../../generated";

type Paragraph = ReadonlyArray<ArchiveTextSegment>;
export type ParagraphPair = readonly [Paragraph | undefined, Paragraph | undefined];

/** A leading paragraph this short is a dateline, not a paragraph of prose. */
const DATELINE_CHARS = 40;

const length = (paragraph: Paragraph): number =>
  paragraph.reduce((total, segment) => total + segment.text.length, 0);

/**
 * Editions break the opening differently: one sets the dateline on its own
 * line, another runs it into the first sentence. When one side has exactly one
 * paragraph more and opens with such a short line, that line is joined to the
 * paragraph after it so the rest pairs up.
 */
const joinDateline = (paragraphs: ReadonlyArray<Paragraph>, other: number): Paragraph[] => {
  const [first, second, ...rest] = paragraphs;
  if (
    first === undefined ||
    second === undefined ||
    paragraphs.length !== other + 1 ||
    length(first) > DATELINE_CHARS
  ) {
    return [...paragraphs];
  }
  return [[...first, { text: " ", hit: false }, ...second], ...rest];
};

/** Rows of paired paragraphs; the shorter edition ends with empty cells. */
export const pairParagraphs = (
  left: ReadonlyArray<Paragraph>,
  right: ReadonlyArray<Paragraph>,
): ParagraphPair[] => {
  const first = joinDateline(left, right.length);
  const second = joinDateline(right, left.length);
  return Array.from({ length: Math.max(first.length, second.length) }, (_, index) => [
    first[index],
    second[index],
  ]);
};
