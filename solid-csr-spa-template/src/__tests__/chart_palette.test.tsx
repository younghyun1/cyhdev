import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, expect, it, vi } from "vitest";
import { createChartPalette } from "../components/chartPalette";

afterEach(() => {
  cleanup();
  document.documentElement.className = "";
  document.documentElement.style.removeProperty("--accent");
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("resolves updated CSS tokens after a theme change and disconnects on cleanup", async () => {
  const disconnect = vi.spyOn(MutationObserver.prototype, "disconnect");
  const removeEventListener = vi.fn();
  vi.stubGlobal("matchMedia", () => ({ addEventListener: vi.fn(), removeEventListener }));
  document.documentElement.style.setProperty("--accent", "#b45309");
  const Probe = () => {
    const palette = createChartPalette();
    return <output>{palette().series}</output>;
  };
  render(() => <Probe />);
  await waitFor(() => expect(screen.getByText("#b45309")).not.toBeNull());
  document.documentElement.style.setProperty("--accent", "#fbbf24");
  document.documentElement.classList.add("dark");
  await waitFor(() => expect(screen.getByText("#fbbf24")).not.toBeNull());
  cleanup();
  expect(disconnect).toHaveBeenCalled();
  expect(removeEventListener).toHaveBeenCalledWith("change", expect.any(Function));
});
