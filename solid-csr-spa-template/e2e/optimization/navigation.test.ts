import { describe, expect, it } from "vitest";
import { assertPageResponse } from "./navigation";

describe("optimization page response diagnostics", () => {
  it("reports document throttling and its bounded numeric retry hint", () => {
    expect(() => assertPageResponse("/verify-email", {
      ok: () => false, status: () => 429, headers: () => ({ "retry-after": "1" }),
    })).toThrow("Page response failed for /verify-email: HTTP 429; Retry-After 1s");
  });

  it("omits arbitrary response headers and malformed retry hints", () => {
    expect(() => assertPageResponse("/reset-password", {
      ok: () => false, status: () => 503,
      headers: () => ({ "retry-after": "https://example.test/?token=private", "set-cookie": "private" }),
    })).toThrowError(new Error("Page response failed for /reset-password: HTTP 503"));
  });

  it("requires a document response and accepts successful responses", () => {
    expect(() => assertPageResponse("/about", null)).toThrow("Page response missing for /about");
    expect(() => assertPageResponse("/about", { ok: () => true, status: () => 200, headers: () => ({}) })).not.toThrow();
  });
});
