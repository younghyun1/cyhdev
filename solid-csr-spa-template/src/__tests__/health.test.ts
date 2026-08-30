import { describe, it, expect } from "vitest";
import { parseUptimeToMs, formatUptimeMs, formatIsoAge } from "../state/health";

describe("parseUptimeToMs", () => {
  it("parses days, hours, minutes, seconds", () => {
    expect(parseUptimeToMs("1 day, 2 hours, 3 minutes, 4 seconds")).toBe(
      1 * 86400000 + 2 * 3600000 + 3 * 60000 + 4 * 1000,
    );
  });

  it("parses single unit", () => {
    expect(parseUptimeToMs("5 hours")).toBe(5 * 3600000);
  });

  it("returns null for undefined", () => {
    expect(parseUptimeToMs(undefined)).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(parseUptimeToMs("")).toBeNull();
  });
});

describe("formatUptimeMs", () => {
  it("formats days", () => {
    expect(formatUptimeMs(90061000)).toBe("1d 1h 1m 1s");
  });

  it("formats hours only", () => {
    expect(formatUptimeMs(3661000)).toBe("1h 1m 1s");
  });

  it("formats minutes only", () => {
    expect(formatUptimeMs(61000)).toBe("1m 1s");
  });

  it("formats seconds only", () => {
    expect(formatUptimeMs(5000)).toBe("5s");
  });

  it("returns dash for null", () => {
    expect(formatUptimeMs(null)).toBe("–");
  });

  it("returns dash for negative", () => {
    expect(formatUptimeMs(-1)).toBe("–");
  });
});

describe("formatIsoAge", () => {
  it("returns dash for undefined", () => {
    expect(formatIsoAge(undefined, null)).toBe("–");
  });

  it("formats seconds ago", () => {
    const now = new Date("2025-01-01T00:00:30Z");
    expect(formatIsoAge("2025-01-01T00:00:00Z", now)).toBe("30s ago");
  });

  it("formats minutes and seconds ago", () => {
    const now = new Date("2025-01-01T00:02:15Z");
    expect(formatIsoAge("2025-01-01T00:00:00Z", now)).toBe("2m 15s ago");
  });
});
