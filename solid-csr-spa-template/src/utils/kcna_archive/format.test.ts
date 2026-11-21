import { describe, expect, it } from "vitest";

import { formatBytes, formatCount, formatDate, formatMonth, formatPercent, formatTimestamp, formatYear } from "./format";

describe("formatting", () => {
  it("writes numbers the way the selected language does", () => {
    expect(formatCount(1234567, "en-US")).toBe("1,234,567");
    expect(formatCount(1234567, "de-DE")).toBe("1.234.567");
  });

  it("formats dates in UTC so the reader's time zone never moves the day", () => {
    expect(formatDate("2026-09-02", "en-US")).toBe("Sep 2, 2026");
    expect(formatMonth("2026-09-01", "en-US")).toBe("Sep 2026");
    expect(formatYear("2026-09-01", "en-US")).toBe("2026");
    expect(formatTimestamp("2026-11-21T08:00:00.000Z", "en-US")).toContain("UTC");
  });

  it("returns unparseable input unchanged", () => {
    expect(formatDate("soon", "en-US")).toBe("soon");
    expect(formatTimestamp("soon", "en-US")).toBe("soon");
    expect(formatMonth("soon", "en-US")).toBe("soon");
  });

  it("formats percentages and memory sizes", () => {
    expect(formatPercent(12.5, "en-US")).toBe("12.50%");
    expect(formatBytes(1_610_612_736, "en-US")).toBe("1.5 GB");
    expect(formatBytes(52_428_800, "en-US")).toBe("50 MB");
  });

  it("falls back to English for a tag the runtime rejects", () => {
    expect(formatCount(1000, "not a locale")).toBe("1,000");
  });
});
