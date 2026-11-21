import { describe, expect, it } from "vitest";

import { linePath, parseTerms, periodLabel, plotted, share, trendsCsv } from "./trends";

const trends = {
  bucket: "year" as const,
  starts: ["2025-01-01", "2026-01-01"],
  totals: [200, 0],
  series: [
    { term: "핵무력", counts: [50, 0] },
    { term: "a,b", counts: [1, 0] },
  ],
};

describe("parseTerms", () => {
  it("splits, trims, deduplicates, and caps at four", () => {
    expect(parseTerms(" 핵무력, 경제 ,, 핵무력")).toEqual(["핵무력", "경제"]);
    expect(parseTerms("a，b、c\nd,e")).toEqual(["a", "b", "c", "d"]);
    expect(parseTerms("  ")).toEqual([]);
  });

  it("cuts a term at the server's length limit", () => {
    expect(parseTerms("x".repeat(150))[0]).toHaveLength(100);
  });
});

describe("plotted values", () => {
  it("computes shares without dividing by zero", () => {
    expect(share(50, 200)).toBe(25);
    expect(share(3, 0)).toBe(0);
    expect(plotted(trends, "share")).toEqual([
      [25, 0],
      [0.5, 0],
    ]);
    expect(plotted(trends, "count")[0]).toEqual([50, 0]);
  });

  it("labels periods by their bucket size", () => {
    expect(periodLabel("2026-03-01", "year")).toBe("2026");
    expect(periodLabel("2026-03-01", "month")).toBe("2026-03");
  });
});

describe("linePath", () => {
  it("scales values into the box with the origin at the bottom", () => {
    expect(linePath([0, 5, 10], 10, 100, 50)).toBe("M0.00 50.00 L50.00 25.00 L100.00 0.00");
    expect(linePath([7], 10, 100, 50)).toBe("M0.00 15.00");
    expect(linePath([], 10, 100, 50)).toBe("");
    expect(linePath([1, 2], 0, 100, 50)).toBe("M0.00 50.00 L100.00 50.00");
  });
});

describe("trendsCsv", () => {
  it("writes one row per period and quotes where needed", () => {
    expect(trendsCsv(trends).split("\r\n")).toEqual([
      'period,documents,핵무력 (count),핵무력 (share %),"a,b (count)","a,b (share %)"',
      "2025,200,50,25.000,1,0.500",
      "2026,0,0,0.000,0,0.000",
    ]);
  });
});
