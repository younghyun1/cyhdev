import { describe, expect, it } from "vitest";
import { parseOidcFragment } from "../services/oidc_fragment";

describe("OIDC callback fragments", () => {
  it("accepts a fixed-size URL-safe link completion token", () => {
    const token = "a".repeat(43);
    expect(parseOidcFragment(`#oidc_link_token=${token}`)).toEqual({
      kind: "link-ready",
      completionToken: token,
    });
  });

  it("rejects malformed completion tokens", () => {
    expect(parseOidcFragment("#oidc_link_token=short")).toEqual({
      kind: "failed",
    });
  });

  it("leaves unrelated anchors outside the OIDC namespace", () => {
    expect(parseOidcFragment("#photograph-7")).toEqual({ kind: "none" });
  });
});
