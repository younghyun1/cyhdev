import { describe, expect, it } from "vitest";

import { ApiContractError } from "../../generated";
import { failureKey, failureKind, isNotReady } from "./failure";

const failed = (status: number, code?: number) =>
  new ApiContractError(status, code === undefined ? "" : JSON.stringify({ error_code: code }));

describe("failureKind", () => {
  it("tells the snapshot states apart by their error codes", () => {
    expect(failureKind(failed(503, 113))).toBe("notInstalled");
    expect(failureKind(failed(503, 114))).toBe("loading");
    expect(failureKind(failed(503, 115))).toBe("loadFailed");
    expect(failureKind(failed(404, 112))).toBe("notFound");
    expect(failureKind(failed(409, 116))).toBe("busy");
    expect(failureKind(failed(500, 117))).toBe("reloadFailed");
  });

  it("falls back to the HTTP status", () => {
    expect(failureKind(failed(404))).toBe("notFound");
    expect(failureKind(failed(400))).toBe("badRequest");
    expect(failureKind(failed(409))).toBe("busy");
    expect(failureKind(failed(503))).toBe("unavailable");
    expect(failureKind(failed(500, 118))).toBe("unavailable");
  });

  it("treats anything that is not an API answer as a network failure", () => {
    expect(failureKind(new TypeError("fetch failed"))).toBe("network");
    expect(failureKind("nope")).toBe("network");
  });

  it("reports a missing, loading, or broken snapshot as a state, not an error", () => {
    expect(isNotReady("notInstalled")).toBe(true);
    expect(isNotReady("loading")).toBe(true);
    expect(isNotReady("loadFailed")).toBe(true);
    expect(isNotReady("network")).toBe(false);
    expect(failureKey("loadFailed")).toBe("kcna.state.failed");
    expect(failureKey("network")).toBe("kcna.failure.network");
  });
});
