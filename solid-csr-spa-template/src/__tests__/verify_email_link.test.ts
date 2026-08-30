import { describe, expect, it } from "vitest";

import { consumeEmailVerificationFragment } from "../pages/verify_email_link";

describe("email verification link consumption", () => {
  it("retains a canonical fragment token only in returned component state", () => {
    const token = "6ba7b810-9dad-41d1-80b4-00c04fd430c8";
    window.history.replaceState(
      { source: "test" },
      "",
      `/verify-email?legacy=discarded#token=${token}`,
    );

    const state = consumeEmailVerificationFragment(
      window.location,
      window.history,
    );

    expect(state).toEqual({ kind: "ready", token });
    expect(window.location.pathname).toBe("/verify-email");
    expect(window.location.search).toBe("");
    expect(window.location.hash).toBe("");
    expect(window.location.href).not.toContain(token);
  });

  it.each([
    ["", "missing"],
    ["#token=not-a-uuid", "invalid"],
    [
      "#token=6ba7b810-9dad-41d1-80b4-00c04fd430c8&source=email",
      "invalid",
    ],
    [
      "#token=6ba7b810-9dad-41d1-80b4-00c04fd430c8&token=6ba7b810-9dad-41d1-80b4-00c04fd430c8",
      "invalid",
    ],
  ] as const)("classifies and scrubs %s", (fragment, expectedKind) => {
    window.history.replaceState(null, "", `/verify-email${fragment}`);

    const state = consumeEmailVerificationFragment(
      window.location,
      window.history,
    );

    expect(state.kind).toBe(expectedKind);
    expect(window.location.hash).toBe("");
  });
});
