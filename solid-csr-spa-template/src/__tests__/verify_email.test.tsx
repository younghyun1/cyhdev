import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { verifyUserEmail } = vi.hoisted(() => ({
  verifyUserEmail: vi.fn(),
}));

vi.mock("@solidjs/router", () => ({
  useNavigate: () => vi.fn(),
}));

vi.mock("../services/all_api", () => ({
  authApi: { verifyUserEmail },
  i18nApi: { getUiTextBundle: vi.fn() },
}));

import VerifyEmailPage from "../pages/verify_email";

describe("email verification confirmation", () => {
  beforeEach(() => {
    verifyUserEmail.mockReset();
    window.history.replaceState(
      null,
      "",
      "/verify-email#token=6ba7b810-9dad-41d1-80b4-00c04fd430c8",
    );
  });

  it("does not post until the user activates the confirmation button", async () => {
    verifyUserEmail.mockResolvedValue({
      success: true,
      data: { verified_at: "2026-08-30T12:00:00Z" },
      meta: {
        time_to_process: "redacted",
        timestamp: "2026-08-30T12:00:00Z",
        metadata: null,
      },
    });
    render(() => <VerifyEmailPage />);

    const button = await screen.findByRole("button", {
      name: "Confirm Email Verification",
    });
    expect(verifyUserEmail).not.toHaveBeenCalled();
    expect(window.location.hash).toBe("");

    fireEvent.click(button);

    await waitFor(() => {
      expect(verifyUserEmail).toHaveBeenCalledWith({
        email_validation_token_id: "6ba7b810-9dad-41d1-80b4-00c04fd430c8",
      });
    });
  });
});
