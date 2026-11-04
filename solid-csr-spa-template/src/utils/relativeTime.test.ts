import { describe, expect, it } from "vitest";
import { formatRelativeTime } from "./relativeTime";

const NOW = Date.parse("2026-11-04T12:00:00Z");

describe("formatRelativeTime", () => {
  it("picks the largest whole unit", () => {
    expect(formatRelativeTime("2026-11-04T11:59:30Z", "en-US", NOW)).toBe("now");
    expect(formatRelativeTime("2026-11-04T11:15:00Z", "en-US", NOW)).toBe("45 minutes ago");
    expect(formatRelativeTime("2026-11-04T09:00:00Z", "en-US", NOW)).toBe("3 hours ago");
    expect(formatRelativeTime("2026-11-03T12:00:00Z", "en-US", NOW)).toBe("yesterday");
    expect(formatRelativeTime("2026-10-01T12:00:00Z", "en-US", NOW)).toBe("last month");
    expect(formatRelativeTime("2024-11-04T12:00:00Z", "en-US", NOW)).toBe("2 years ago");
  });

  it("formats in the requested locale and rejects bad input", () => {
    expect(formatRelativeTime("2026-11-04T09:00:00Z", "ko-KR", NOW)).toBe("3시간 전");
    expect(formatRelativeTime("not a date", "en-US", NOW)).toBe("");
  });
});
