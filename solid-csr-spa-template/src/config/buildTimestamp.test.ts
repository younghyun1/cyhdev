import { describe, expect, it } from "vitest";

import {
  resolveBuildTimestamp,
  resolveConfiguredBuildTimestamp,
} from "./buildTimestamp";

describe("resolveBuildTimestamp", () => {
  it("converts SOURCE_DATE_EPOCH seconds to UTC", () => {
    expect(resolveBuildTimestamp("1788120473")).toBe("2026-08-30T20:07:53.000Z");
  });

  it("uses the supplied compilation time when no source epoch exists", () => {
    const compilationTime = new Date("2026-08-30T21:00:00.000Z");
    expect(resolveBuildTimestamp(undefined, compilationTime)).toBe(
      "2026-08-30T21:00:00.000Z",
    );
  });

  it("rejects malformed and unsafe epochs", () => {
    expect(() => resolveBuildTimestamp("-1")).toThrow(/unsigned integer/);
    expect(() => resolveBuildTimestamp("tomorrow")).toThrow(/unsigned integer/);
    expect(() => resolveBuildTimestamp("9223372036854775807")).toThrow(/safe integer/);
  });

  it("prefers the cache-neutral application build epoch", () => {
    expect(
      resolveConfiguredBuildTimestamp("1788120473", "1788120000"),
    ).toBe("2026-08-30T20:07:53.000Z");
  });

  it("retains SOURCE_DATE_EPOCH as a direct-build fallback", () => {
    expect(resolveConfiguredBuildTimestamp(undefined, "1788120473")).toBe(
      "2026-08-30T20:07:53.000Z",
    );
  });
});
