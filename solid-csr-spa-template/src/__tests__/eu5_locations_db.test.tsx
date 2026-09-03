import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@solidjs/router", () => ({
  useLocation: () => ({ pathname: "/" }),
}));

vi.mock("../services/all_api", () => ({
  authApi: { logout: vi.fn() },
  i18nApi: { getUiTextBundle: vi.fn() },
}));

import PublicNavigation, {
  NAV_ITEMS,
  NAV_LINKS,
} from "../components/PublicNavigation";
import TopBar from "../components/TopBar";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { KO_KR_DEFAULT_TEXTS } from "../i18n/defaults/ko-kr";
import Eu5LocationsDb, {
  EU5_THEME_READY_MESSAGE,
  calculateViewportOffsets,
  serializeEu5Theme,
} from "../pages/eu5_locations_db";
import { setAuthenticated } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";
import { setTheme } from "../state/theme";

describe("EU5 Locations DB page", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
    setTheme("light");
  });

  afterEach(() => cleanup());

  it("calculates the remaining viewport between the measured site bars", () => {
    expect(calculateViewportOffsets(63.2, 940.4, 984)).toEqual({
      top: 64,
      bottom: 44,
    });
  });

  it("loads the first-party Slint host eagerly with browser permissions", () => {
    const result = render(() => <Eu5LocationsDb />);
    const frame = result.container.querySelector("iframe");

    expect(frame?.getAttribute("src")).toBe(
      "/eu5-locations-db/app/index.html",
    );
    expect(frame?.getAttribute("title")).toBe("EU5 Locations DB");
    expect(frame?.getAttribute("loading")).toBe("eager");
    expect(frame?.getAttribute("sandbox")).toContain("allow-scripts");
    expect(frame?.getAttribute("sandbox")).toContain("allow-same-origin");
    expect(frame?.getAttribute("sandbox")).toContain("allow-popups");
  });

  it("serializes the bounded iframe theme protocol", () => {
    expect(serializeEu5Theme("light")).toBe("cyhdev:eu5-theme:light");
    expect(serializeEu5Theme("dark")).toBe("cyhdev:eu5-theme:dark");
  });

  it("sends the current theme when the iframe loads and whenever it changes", async () => {
    const result = render(() => <Eu5LocationsDb />);
    const frame = result.container.querySelector("iframe");
    const target = frame?.contentWindow;
    expect(frame).not.toBeNull();
    expect(target).not.toBeNull();
    if (!frame || !target) return;
    const postMessage = vi.spyOn(target, "postMessage");

    fireEvent.load(frame);
    expect(postMessage).toHaveBeenCalledWith(
      "cyhdev:eu5-theme:light",
      window.location.origin,
    );

    setTheme("dark");
    await waitFor(() => {
      expect(postMessage).toHaveBeenCalledWith(
        "cyhdev:eu5-theme:dark",
        window.location.origin,
      );
    });
  });

  it("answers only a same-origin ready message from its own iframe", () => {
    const result = render(() => <Eu5LocationsDb />);
    const frame = result.container.querySelector("iframe");
    const target = frame?.contentWindow;
    expect(target).not.toBeNull();
    if (!target) return;
    const postMessage = vi.spyOn(target, "postMessage");

    window.dispatchEvent(
      new MessageEvent("message", {
        data: EU5_THEME_READY_MESSAGE,
        origin: "https://invalid.example",
        source: target,
      }),
    );
    window.dispatchEvent(
      new MessageEvent("message", {
        data: "unrelated-message",
        origin: window.location.origin,
        source: target,
      }),
    );
    expect(postMessage).not.toHaveBeenCalled();

    window.dispatchEvent(
      new MessageEvent("message", {
        data: EU5_THEME_READY_MESSAGE,
        origin: window.location.origin,
        source: target,
      }),
    );
    expect(postMessage).toHaveBeenCalledOnce();
    expect(postMessage).toHaveBeenCalledWith(
      "cyhdev:eu5-theme:light",
      window.location.origin,
    );
  });

  it("keeps the exact English label and a Korean translation", () => {
    expect(EN_US_DEFAULT_TEXTS["top_bar.nav.eu5_locations_db"]).toBe(
      "EU5 Locations DB",
    );
    expect(KO_KR_DEFAULT_TEXTS["top_bar.nav.eu5_locations_db"]).toBe(
      "EU5 위치 데이터베이스",
    );
  });

  it("shows the exact public label in desktop and logged-out mobile navigation", async () => {
    expect(NAV_LINKS).toContainEqual({
      href: "/eu5-locations-db",
      labelKey: "top_bar.nav.eu5_locations_db",
    });
    expect(
      NAV_ITEMS.filter((item) => item.kind === "group").map(
        (item) => item.labelKey,
      ),
    ).toEqual([
      "top_bar.nav.about_group",
      "top_bar.nav.community_group",
      "top_bar.nav.projects_group",
    ]);

    render(() => <TopBar />);
    fireEvent.click(screen.getByRole("button", { name: "Projects" }));
    expect(
      await screen.findByRole("link", { name: "EU5 Locations DB" }),
    ).not.toBeNull();

    cleanup();
    render(() => (
      <ul>
        <PublicNavigation variant="mobile" isActive={() => false} />
      </ul>
    ));
    fireEvent.click(screen.getByRole("button", { name: "Projects" }));
    expect(
      await screen.findByRole("link", { name: "EU5 Locations DB" }),
    ).not.toBeNull();
  });

  it("keeps one dropdown open and closes it after navigation or outside input", async () => {
    render(() => (
      <ul>
        <PublicNavigation variant="desktop" isActive={() => false} />
      </ul>
    ));

    fireEvent.click(screen.getByRole("button", { name: "About" }));
    expect(await screen.findByRole("link", { name: "About Me" })).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Community" }));
    await waitFor(() => {
      expect(screen.queryByRole("link", { name: "About Me" })).toBeNull();
      expect(screen.queryByRole("link", { name: "Forum" })).not.toBeNull();
    });

    const forumLink = screen.getByRole("link", { name: "Forum" });
    forumLink.addEventListener("click", (event) => event.preventDefault(), {
      once: true,
    });
    fireEvent.click(forumLink);
    await waitFor(() => {
      expect(screen.queryByRole("link", { name: "Forum" })).toBeNull();
    });

    fireEvent.click(screen.getByRole("button", { name: "Projects" }));
    expect(
      await screen.findByRole("link", { name: "EU5 Locations DB" }),
    ).not.toBeNull();
    fireEvent.pointerDown(document.body);
    await waitFor(() => {
      expect(
        screen.queryByRole("link", { name: "EU5 Locations DB" }),
      ).toBeNull();
    });

    fireEvent.click(screen.getByRole("button", { name: "About" }));
    await screen.findByRole("link", { name: "About Me" });
    fireEvent.keyDown(document, { key: "Escape" });
    await waitFor(() => {
      expect(screen.queryByRole("link", { name: "About Me" })).toBeNull();
    });
  });
});
