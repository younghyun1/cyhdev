import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ApiContractError } from "../generated";

const { login, navigate } = vi.hoisted(() => ({
  login: vi.fn(),
  navigate: vi.fn(),
}));

vi.mock("@solidjs/router", () => ({
  useNavigate: () => navigate,
  useSearchParams: () => [{}],
}));

vi.mock("../services/all_api", () => ({
  authApi: { login, me: vi.fn(), isSuperuser: vi.fn() },
  oidcApi: { status: vi.fn().mockResolvedValue({ data: { enabled: false } }) },
  i18nApi: { getUiTextBundle: vi.fn() },
}));

import LoginPage from "../pages/login";

describe("login for an unverified account", () => {
  beforeEach(() => {
    login.mockReset();
    navigate.mockReset();
    window.history.replaceState(null, "", "/login");
  });

  it("shows the verification state and the signup path for a new link", async () => {
    login.mockRejectedValue(
      new ApiContractError(
        403,
        JSON.stringify({ success: false, error_code: 68, message: "Verify first" }),
      ),
    );
    render(() => <LoginPage />);

    fireEvent.input(screen.getByPlaceholderText("Email"), {
      target: { value: "owner@example.test" },
    });
    fireEvent.input(screen.getByPlaceholderText("Password"), {
      target: { value: "ValidPass123" },
    });
    fireEvent.submit(screen.getByRole("button", { name: "Login" }));

    expect(
      await screen.findByText(/Verify your email address before logging in/),
    ).toBeTruthy();
    expect(screen.queryByText("Login failed")).toBeNull();
    fireEvent.click(
      screen.getByRole("button", { name: "Sign up again for a new link" }),
    );
    expect(navigate).toHaveBeenCalledWith("/register");
  });

  it("keeps the generic failure for rejected credentials", async () => {
    login.mockRejectedValue(new ApiContractError(401, "{}"));
    render(() => <LoginPage />);

    fireEvent.input(screen.getByPlaceholderText("Email"), {
      target: { value: "owner@example.test" },
    });
    fireEvent.input(screen.getByPlaceholderText("Password"), {
      target: { value: "WrongPass123" },
    });
    fireEvent.submit(screen.getByRole("button", { name: "Login" }));

    expect(await screen.findByText("Login failed")).toBeTruthy();
    expect(screen.queryByText(/Verify your email address/)).toBeNull();
  });
});
