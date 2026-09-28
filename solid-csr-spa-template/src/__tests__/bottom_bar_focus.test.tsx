import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import BottomBar from "../components/BottomBar";

const environment = vi.hoisted(() => ({ mobile: true }));
vi.mock("@solidjs/router", () => ({ useLocation: () => ({ pathname: "/minecraft" }) }));
vi.mock("../utils/mediaQuery", () => ({ createMediaQuery: () => () => environment.mobile }));
vi.mock("../state/health", () => ({ healthState: () => null, clientNow: () => new Date(0), setClientNow: vi.fn(), refreshHealthStateIfStale: vi.fn(), formatIsoAge: () => "0s" }));
vi.mock("../components/SystemStatusDetails", () => ({ default: () => null, createLiveUptime: () => () => "0s" }));
vi.mock("../components/BuildDetails", () => ({ default: () => null }));

describe("mobile status footer focus", () => {
  beforeEach(() => { environment.mobile = true; });
  afterEach(() => cleanup());
  const setup = () => {
    const submit = vi.fn();
    const view = render(() => <><input aria-label="Example field" /><button type="button" onClick={submit}>Submit example</button><BottomBar /></>);
    const footer = view.container.querySelector("footer");
    if (!footer) throw new Error("Missing footer fixture");
    return { footer, input: screen.getByLabelText("Example field"), button: screen.getByRole("button", { name: "Submit example" }), submit };
  };

  it("keeps the footer hidden through pointerdown, blur, and the completing click", async () => {
    const { footer, input, button, submit } = setup();
    input.focus(); await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBe("true"));
    fireEvent.pointerDown(button); button.focus();
    await Promise.resolve();
    expect(footer.getAttribute("aria-hidden")).toBe("true");
    fireEvent.pointerUp(button); fireEvent.click(button);
    expect(submit).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBeNull());
  });

  it("restores the footer after a cancelled pointer without submitting", async () => {
    const { footer, input, button, submit } = setup();
    input.focus(); await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBe("true"));
    fireEvent.pointerDown(button); button.focus(); await Promise.resolve();
    fireEvent.pointerCancel(button);
    await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBeNull());
    expect(submit).not.toHaveBeenCalled();
  });

  it("restores immediately after keyboard focus leaves a field", async () => {
    const { footer, input, button } = setup();
    input.focus(); await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBe("true"));
    button.focus(); await waitFor(() => expect(footer.getAttribute("aria-hidden")).toBeNull());
  });

  it("leaves desktop status visible while editing", async () => {
    environment.mobile = false; const { footer, input } = setup();
    input.focus(); await Promise.resolve();
    expect(footer.getAttribute("aria-hidden")).toBeNull();
  });
});
