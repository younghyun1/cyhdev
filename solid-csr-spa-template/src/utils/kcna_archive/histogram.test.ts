import { describe, expect, it } from "vitest";

import { isoDate, monthEnd, monthIndex, monthStart, parseDateJump } from "./dates";
import { barAt, barHeight, inRange, monthBars, rangeOf, yearTicks } from "./histogram";

const buckets = [
  { start: "2025-11-01", count: 4 },
  { start: "2026-02-01", count: 100 },
];

describe("dates", () => {
  it("converts between dates and month indexes", () => {
    expect(monthStart(monthIndex("2026-11-21"))).toBe("2026-11-01");
    expect(monthEnd(monthIndex("2024-02-10"))).toBe("2024-02-29");
    expect(monthEnd(monthIndex("2026-12-01"))).toBe("2026-12-31");
    expect(monthIndex("2027-01-01") - monthIndex("2026-12-01")).toBe(1);
    expect(isoDate(2026, 3, 5)).toBe("2026-03-05");
  });
});

describe("monthBars", () => {
  it("fills gaps between the first and last bucket", () => {
    const bars = monthBars(buckets);
    expect(bars.map((bar) => [bar.start, bar.count])).toEqual([
      ["2025-11-01", 4],
      ["2025-12-01", 0],
      ["2026-01-01", 0],
      ["2026-02-01", 100],
    ]);
    expect(monthBars([])).toEqual([]);
  });
});

describe("bar geometry", () => {
  it("scales heights by square root with a visible minimum", () => {
    expect(barHeight(100, 100)).toBe(1);
    expect(barHeight(25, 100)).toBe(0.5);
    expect(barHeight(0, 100)).toBe(0);
    expect(barHeight(1, 1_000_000)).toBe(0.02);
    expect(barHeight(5, 0)).toBe(0);
  });

  it("finds the bar under a pointer position", () => {
    const bars = monthBars(buckets);
    expect(barAt(bars, 0)?.start).toBe("2025-11-01");
    expect(barAt(bars, 0.49)?.start).toBe("2025-12-01");
    expect(barAt(bars, 1)?.start).toBe("2026-02-01");
    expect(barAt(bars, -3)?.start).toBe("2025-11-01");
    expect(barAt([], 0.5)).toBeUndefined();
  });
});

describe("selection", () => {
  it("covers whole months in either drag direction", () => {
    const bars = monthBars(buckets);
    const first = bars[0];
    const last = bars[3];
    if (first === undefined || last === undefined) throw new Error("bars are missing");
    expect(rangeOf(last, first)).toEqual({ from: "2025-11-01", to: "2026-02-28" });
    expect(rangeOf(first, first)).toEqual({ from: "2025-11-01", to: "2025-11-30" });
  });

  it("tests bars against open and closed ranges", () => {
    const bar = monthBars(buckets)[1];
    if (bar === undefined) throw new Error("bar is missing");
    expect(inRange(bar, null, null)).toBe(true);
    expect(inRange(bar, "2025-12-15", null)).toBe(true);
    expect(inRange(bar, "2026-01-01", null)).toBe(false);
    expect(inRange(bar, null, "2025-11-30")).toBe(false);
  });

  it("thins year ticks to the available slots", () => {
    const years = Array.from({ length: 30 }, (_, index) => ({ start: `${1997 + index}-01-01`, count: 1 }));
    const bars = monthBars(years);
    expect(yearTicks(bars, 100)).toHaveLength(30);
    expect(yearTicks(bars, 10).map((bar) => bar.start.slice(0, 4))).toEqual([
      "1997", "2000", "2003", "2006", "2009", "2012", "2015", "2018", "2021", "2024",
    ]);
  });
});

describe("parseDateJump", () => {
  it("reads years, months, and days", () => {
    expect(parseDateJump("2026")).toEqual({ from: "2026-01-01", to: "2026-12-31", label: "2026" });
    expect(parseDateJump("2024-02")).toEqual({ from: "2024-02-01", to: "2024-02-29", label: "2024-02" });
    expect(parseDateJump(" 2026.11.21. ")).toEqual({ from: "2026-11-21", to: "2026-11-21", label: "2026-11-21" });
    expect(parseDateJump("2026/9")).toEqual({ from: "2026-09-01", to: "2026-09-30", label: "2026-09" });
  });

  it("rejects text and impossible dates", () => {
    for (const input of ["", "김정은", "1800", "2026-13", "2026-02-30", "20261121"]) {
      expect(parseDateJump(input)).toBeUndefined();
    }
  });
});
