import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { Show, createSignal } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MobileDialog } from "../components/MobileDialog";
import { createMediaQuery } from "../utils/mediaQuery";

afterEach(() => {
  cleanup();
  document.body.style.overflow = "";
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("mobile foundations", () => {
  it("reacts to matchMedia changes", async () => {
    let listener: ((event: MediaQueryListEvent) => void) | undefined;
    const media = {
      matches: false,
      media: "(max-width: 767px)",
      addEventListener: vi.fn((_name: string, next: EventListener) => {
        listener = next as (event: MediaQueryListEvent) => void;
      }),
      removeEventListener: vi.fn(),
    } as unknown as MediaQueryList;
    vi.stubGlobal("matchMedia", vi.fn().mockReturnValue(media));

    const Probe = () => {
      const mobile = createMediaQuery("(max-width: 767px)");
      return <output>{mobile() ? "mobile" : "desktop"}</output>;
    };
    render(() => <Probe />);
    expect(screen.getByText("desktop")).not.toBeNull();
    listener?.({ matches: true } as MediaQueryListEvent);
    await waitFor(() => expect(screen.getByText("mobile")).not.toBeNull());
  });

  it("traps focus, locks scrolling, closes on Escape, and restores focus", async () => {
    const Example = () => {
      const [open, setOpen] = createSignal(false);
      return (
        <>
          <div id="app-root">
            <button type="button" onClick={() => setOpen(true)}>
              Open
            </button>
          </div>
          <Show when={open()}>
            <MobileDialog
              onClose={() => setOpen(false)}
              overlayClass="overlay"
              panelClass="panel"
              ariaLabel="Example"
            >
              <button type="button">First</button>
              <button type="button">Last</button>
            </MobileDialog>
          </Show>
        </>
      );
    };

    render(() => <Example />);
    const opener = screen.getByRole("button", { name: "Open" });
    opener.focus();
    fireEvent.click(opener);
    await waitFor(() =>
      expect(screen.getByRole("dialog", { name: "Example" })).not.toBeNull(),
    );
    expect(document.body.style.overflow).toBe("hidden");
    expect(document.getElementById("app-root")?.inert).toBe(true);

    const first = screen.getByRole("button", { name: "First" });
    const last = screen.getByRole("button", { name: "Last" });
    last.focus();
    fireEvent.keyDown(document, { key: "Tab" });
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Example" })).toBeNull(),
    );
    expect(document.body.style.overflow).toBe("");
    await waitFor(() => expect(document.activeElement).toBe(opener));
  });
});
