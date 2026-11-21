import { describe, expect, it } from "vitest";

import type { ArchiveTextSegment } from "../../generated";
import { pairParagraphs } from "./parallel";
import { plainText } from "./segments";

const paragraph = (text: string): ArchiveTextSegment[] => [{ text, hit: false }];
const text = (cell: ReadonlyArray<ArchiveTextSegment> | undefined): string | undefined =>
  cell === undefined ? undefined : plainText(cell);

describe("pairParagraphs", () => {
  it("pairs equal counts by position", () => {
    const rows = pairParagraphs([paragraph("a"), paragraph("b")], [paragraph("가"), paragraph("나")]);
    expect(rows.map(([left, right]) => [text(left), text(right)])).toEqual([
      ["a", "가"],
      ["b", "나"],
    ]);
  });

  it("joins a standalone dateline to the paragraph after it", () => {
    const english = [paragraph("Pyongyang, November 21 (KCNA) -- The exhibition opened."), paragraph("Second.")];
    const korean = [paragraph("(평양 11월 21일발 조선중앙통신)"), paragraph("전시회가 개막되였다."), paragraph("둘째.")];
    const rows = pairParagraphs(english, korean);
    expect(rows).toHaveLength(2);
    expect(text(rows[0]?.[1])).toBe("(평양 11월 21일발 조선중앙통신) 전시회가 개막되였다.");
    expect(text(rows[1]?.[1])).toBe("둘째.");
  });

  it("leaves other differences alone and pads the shorter side", () => {
    const long = "가".repeat(60);
    const rows = pairParagraphs([paragraph("a")], [paragraph(long), paragraph("나"), paragraph("다")]);
    expect(rows).toHaveLength(3);
    expect(text(rows[0]?.[1])).toBe(long);
    expect(rows[2]?.[0]).toBeUndefined();
    expect(pairParagraphs([], [])).toEqual([]);
  });

  it("keeps the match marks of both editions", () => {
    const marked = [[{ text: "다리 ", hit: false }, { text: "준공", hit: true }]];
    const rows = pairParagraphs(marked, marked);
    expect(rows[0]?.[0]?.[1]?.hit).toBe(true);
  });
});
