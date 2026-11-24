import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const router = vi.hoisted(() => ({
  navigate: vi.fn(),
  location: { pathname: "/eu5-locations-db", search: "", hash: "" },
}));

vi.mock("@solidjs/router", () => ({
  useLocation: () => router.location,
  useNavigate: () => router.navigate,
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
  EU5_DEFAULT_PALETTES,
  EU5_PALETTE_PROPERTIES,
  EU5_PALETTE_TOKENS,
  EU5_VIEW_STATE_MAX_LENGTH,
  type Eu5HostTheme,
  buildEu5HostMessage,
  calculateViewportOffsets,
  createEu5ColorReader,
  isEu5Ready,
  isEu5ViewState,
  normalizeCssColor,
  readEu5StateMessage,
  readEu5ViewStateParam,
  resolveEu5Theme,
  sameEu5HostSettings,
  withEu5ViewStateParam,
} from "../pages/eu5_locations_db";
import { setAuthenticated } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";
import { setMotionSignal } from "../state/motion";
import { setTheme } from "../state/theme";

const lightTheme: Eu5HostTheme = { mode: "light", ...EU5_DEFAULT_PALETTES.light };
const darkTheme: Eu5HostTheme = { mode: "dark", ...EU5_DEFAULT_PALETTES.dark };
const frameWindow = {} as MessageEventSource;
const readyEvent = {
  data: { type: "cyhdev:eu5-ready", v: 2 },
  origin: "null",
  source: frameWindow,
};

/** Mounts the page and returns the frame window with its postMessage spy. */
const mountPage = () => {
  const result = render(() => <Eu5LocationsDb />);
  const target = result.container.querySelector("iframe")?.contentWindow;
  if (!target) throw new Error("iframe has no content window");
  return { target, postMessage: vi.spyOn(target, "postMessage") };
};

const send = (target: Window, data: unknown, origin = "null"): void => {
  window.dispatchEvent(new MessageEvent("message", { data, origin, source: target }));
};

describe("EU5 Locations DB page", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
    setTheme("light");
    setMotionSignal(true);
    document.documentElement.classList.remove("dark");
    router.navigate.mockClear();
    router.location.search = "";
    router.location.hash = "";
  });

  afterEach(() => cleanup());

  it("calculates the remaining viewport between the measured site bars", () => {
    expect(calculateViewportOffsets(63.2, 940.4, 984)).toEqual({
      top: 64,
      bottom: 44,
    });
  });

  it("loads the host eagerly in an opaque-origin sandbox", () => {
    const result = render(() => <Eu5LocationsDb />);
    const frame = result.container.querySelector("iframe");

    expect(frame?.getAttribute("src")).toBe("/eu5-locations-db/app/index.html");
    expect(frame?.getAttribute("title")).toBe("EU5 Locations DB");
    expect(frame?.getAttribute("loading")).toBe("eager");
    expect(frame?.getAttribute("sandbox")).toContain("allow-scripts");
    expect(frame?.getAttribute("sandbox")).not.toContain("allow-same-origin");
    expect(frame?.getAttribute("sandbox")).toContain("allow-popups");
  });

  it("maps every token to a site custom property and a default for both modes", () => {
    expect(Object.keys(EU5_PALETTE_PROPERTIES)).toEqual([...EU5_PALETTE_TOKENS]);
    expect(EU5_PALETTE_PROPERTIES.inkMuted).toBe("--ink-muted");
    for (const mode of ["light", "dark"] as const) {
      for (const token of EU5_PALETTE_TOKENS) {
        expect(EU5_DEFAULT_PALETTES[mode][token]).toMatch(/^#[0-9a-f]{6}$/);
      }
    }
  });

  it("normalizes CSS colours and rejects anything not opaque sRGB", () => {
    const cases: ReadonlyArray<readonly [string, string | null]> = [
      ["#b45309", "#b45309"],
      ["#F72", "#ff7722"],
      ["#B45309FF", "#b45309"],
      ["#b4530980", null],
      ["  #ABCDEF ", "#abcdef"],
      ["rgb(180, 83, 9)", "#b45309"],
      ["RGB(180 83 9 / 1)", "#b45309"],
      ["rgba(180, 83, 9, 1)", "#b45309"],
      ["rgb(180 83 9 / 100%)", "#b45309"],
      ["rgb(179.6, 83, 9)", "#b45309"],
      ["rgba(0, 0, 0, 0.5)", null],
      ["rgb(0 0 0 / 50%)", null],
      ["rgb(256, 0, 0)", null],
      ["rgb(-1, 0, 0)", null],
      ["rgb(1, 2)", null],
      ["color(srgb 1 0 0)", "#ff0000"],
      ["color(srgb 1 0 0 / 1)", "#ff0000"],
      ["color(srgb 1 0 0 / 0.5)", null],
      ["color(srgb 1.2 0 0)", null],
      ["oklab(0.6 0.1 0.1)", null],
      ["#12", null],
      ["", null],
    ];
    for (const [input, expected] of cases) {
      expect(normalizeCssColor(input), input).toBe(expected);
    }
  });

  it("resolves tokens through the reader and defaults the unreadable ones", () => {
    const reads: string[] = [];
    const theme = resolveEu5Theme("dark", (property) => {
      reads.push(property);
      if (property === "--accent") return "rgb(1, 2, 3)";
      if (property === "--ink") return "rgba(1, 2, 3, 0.5)";
      if (property === "--line") return "not a colour";
      return null;
    });
    expect(reads).toEqual(EU5_PALETTE_TOKENS.map((token) => EU5_PALETTE_PROPERTIES[token]));
    expect(theme.mode).toBe("dark");
    expect(theme.accent).toBe("#010203");
    expect(theme.ink).toBe(EU5_DEFAULT_PALETTES.dark.ink);
    expect(theme.line).toBe(EU5_DEFAULT_PALETTES.dark.line);
    expect(Object.keys(theme)).toEqual(["mode", ...EU5_PALETTE_TOKENS]);
  });

  it("reads site tokens through a probe that it removes again", () => {
    const root = document.documentElement;
    root.style.setProperty("--eu5-test-colour", "rgb(10, 20, 30)");
    const before = document.body.childElementCount;
    const reader = createEu5ColorReader(document);
    // jsdom may not resolve var() in computed styles, so only the shape is fixed.
    expect([null, "#0a141e"]).toContain(reader("--eu5-test-colour"));
    // A property the site does not define must not inherit the body colour.
    expect(reader("--eu5-missing-token")).toBeNull();
    expect(document.body.childElementCount).toBe(before);
    root.style.removeProperty("--eu5-test-colour");
  });

  it("validates view-state charset and length", () => {
    expect(isEu5ViewState("s1")).toBe(true);
    expect(isEu5ViewState("s1~q.new!20york~k.land*sea~lm.2*3")).toBe(true);
    expect(isEu5ViewState("a".repeat(EU5_VIEW_STATE_MAX_LENGTH))).toBe(true);
    expect(isEu5ViewState("a".repeat(EU5_VIEW_STATE_MAX_LENGTH + 1))).toBe(false);
    for (const bad of ["", "s1 q", "s1%20q", "s1+q", "s1/q", "s1\n", "서울", null, 1, {}]) {
      expect(isEu5ViewState(bad), String(bad)).toBe(false);
    }
  });

  it("builds host messages with a derived direction and a guarded view state", () => {
    const message = buildEu5HostMessage({
      theme: darkTheme,
      locale: "ar",
      reducedMotion: true,
      viewState: "s1~q.river",
    });
    expect(message).toEqual({
      type: "cyhdev:eu5-host",
      v: 2,
      theme: darkTheme,
      locale: "ar",
      dir: "rtl",
      reducedMotion: true,
      viewState: "s1~q.river",
    });
    for (const viewState of [undefined, null, "", "bad state"]) {
      const plain = buildEu5HostMessage({
        theme: lightTheme,
        locale: "ko-KR",
        reducedMotion: false,
        viewState,
      });
      expect(plain.dir).toBe("ltr");
      expect("viewState" in plain).toBe(false);
    }
  });

  it("compares host settings without the view state", () => {
    const base = buildEu5HostMessage({
      theme: lightTheme,
      locale: "en-US",
      reducedMotion: false,
      viewState: "s1",
    });
    const without = buildEu5HostMessage({
      theme: lightTheme,
      locale: "en-US",
      reducedMotion: false,
    });
    expect(sameEu5HostSettings(base, without)).toBe(true);
    const variants = [
      buildEu5HostMessage({ theme: darkTheme, locale: "en-US", reducedMotion: false }),
      buildEu5HostMessage({ theme: { ...lightTheme, accent: "#123456" }, locale: "en-US", reducedMotion: false }),
      buildEu5HostMessage({ theme: lightTheme, locale: "ar", reducedMotion: false }),
      buildEu5HostMessage({ theme: lightTheme, locale: "en-US", reducedMotion: true }),
    ];
    for (const variant of variants) {
      expect(sameEu5HostSettings(base, variant)).toBe(false);
    }
  });

  it("identifies ready and state messages by source window, opaque origin, type and version", () => {
    expect(isEu5Ready(readyEvent, frameWindow)).toBe(true);
    expect(isEu5Ready(readyEvent, null)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, source: null }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, source: {} as MessageEventSource }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, origin: "https://cyhdev.com" }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, data: { type: "cyhdev:eu5-ready", v: 1 } }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, data: { type: "cyhdev:eu5-state", v: 2 } }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, data: "cyhdev:eu5-theme-ready" }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, data: [readyEvent.data] }, frameWindow)).toBe(false);
    expect(isEu5Ready({ ...readyEvent, data: null }, frameWindow)).toBe(false);

    const state = (value: unknown) => ({
      ...readyEvent,
      data: { type: "cyhdev:eu5-state", v: 2, state: value },
    });
    expect(readEu5StateMessage(state("s1~q.river"), frameWindow)).toBe("s1~q.river");
    expect(readEu5StateMessage(state("bad state"), frameWindow)).toBeNull();
    expect(readEu5StateMessage(state("a".repeat(2001)), frameWindow)).toBeNull();
    expect(readEu5StateMessage(state(5), frameWindow)).toBeNull();
    expect(readEu5StateMessage({ ...state("s1"), origin: "https://cyhdev.com" }, frameWindow)).toBeNull();
    expect(readEu5StateMessage({ ...state("s1"), data: { type: "cyhdev:eu5-ready", v: 2, state: "s1" } }, frameWindow)).toBeNull();
  });

  it("reads the view parameter from a search string", () => {
    expect(readEu5ViewStateParam("?a=1&view=s1~q.river")).toBe("s1~q.river");
    expect(readEu5ViewStateParam("?view=s1%7Eq.river")).toBe("s1~q.river");
    expect(readEu5ViewStateParam("")).toBeNull();
    expect(readEu5ViewStateParam("?view=")).toBeNull();
    expect(readEu5ViewStateParam("?view=has%20space")).toBeNull();
    expect(readEu5ViewStateParam(`?view=${"a".repeat(2001)}`)).toBeNull();
  });

  it("writes the view parameter last and unencoded, and removes it for the default", () => {
    expect(withEu5ViewStateParam("", "s1~q.river")).toBe("?view=s1~q.river");
    expect(withEu5ViewStateParam("?a=1&view=old", "s1")).toBe("?a=1");
    expect(withEu5ViewStateParam("?view=x", null)).toBe("");
    expect(withEu5ViewStateParam("?a=1", "bad state")).toBe("?a=1");
    expect(withEu5ViewStateParam("?view=old&a=1", "s1~k.land*sea~lm.2*3!")).toBe(
      "?a=1&view=s1~k.land*sea~lm.2*3!",
    );
  });

  it("answers each ready from the frame with a full host message and the view state", () => {
    router.location.search = "?view=s1~q.river";
    const { target, postMessage } = mountPage();

    send(target, { type: "cyhdev:eu5-ready", v: 2 });
    expect(postMessage).toHaveBeenCalledOnce();
    const [message, targetOrigin] = postMessage.mock.calls[0] ?? [];
    expect(targetOrigin).toBe("*");
    expect(message).toEqual(
      buildEu5HostMessage({
        theme: lightTheme,
        locale: "en-US",
        reducedMotion: false,
        viewState: "s1~q.river",
      }),
    );

    // A reload sends a new ready and gets the full message again.
    send(target, { type: "cyhdev:eu5-ready", v: 2 });
    expect(postMessage).toHaveBeenCalledTimes(2);
  });

  it("sends the right-to-left direction and the motion switch for the site settings", () => {
    setLocaleSignal("ar");
    setMotionSignal(false);
    const { target, postMessage } = mountPage();
    send(target, { type: "cyhdev:eu5-ready", v: 2 });
    const message = postMessage.mock.calls[0]?.[0] as { dir: string; reducedMotion: boolean; viewState?: string };
    expect(message.dir).toBe("rtl");
    expect(message.reducedMotion).toBe(true);
    expect("viewState" in message).toBe(false);
  });

  it("ignores ready messages that fail the guard", () => {
    const { target, postMessage } = mountPage();
    const ready = { type: "cyhdev:eu5-ready", v: 2 };
    send(target, ready, "https://invalid.example");
    // A same-origin sender is not the sandboxed app.
    send(target, ready, window.location.origin);
    send(target, { type: "cyhdev:eu5-ready", v: 1 });
    send(target, "cyhdev:eu5-theme-ready");
    window.dispatchEvent(new MessageEvent("message", { data: ready, origin: "null", source: window }));
    expect(postMessage).not.toHaveBeenCalled();
  });

  it("mirrors state messages into the view parameter with history replace", () => {
    router.location.search = "?a=1";
    router.location.hash = "#top";
    const { target, postMessage } = mountPage();

    send(target, { type: "cyhdev:eu5-state", v: 2, state: "s1~q.river~k.land*sea" });
    expect(router.navigate).toHaveBeenCalledWith(
      "/eu5-locations-db?a=1&view=s1~q.river~k.land*sea#top",
      { replace: true, scroll: false },
    );
    // State messages never trigger a host message.
    expect(postMessage).not.toHaveBeenCalled();
  });

  it("removes the parameter for the default state and skips unchanged or invalid ones", () => {
    router.location.search = "?a=1&view=s1~q.river";
    const { target } = mountPage();

    send(target, { type: "cyhdev:eu5-state", v: 2, state: "s1" });
    expect(router.navigate).toHaveBeenCalledWith("/eu5-locations-db?a=1", {
      replace: true,
      scroll: false,
    });
    router.navigate.mockClear();

    send(target, { type: "cyhdev:eu5-state", v: 2, state: "s1~q.river" });
    expect(router.navigate).not.toHaveBeenCalled();
    send(target, { type: "cyhdev:eu5-state", v: 2, state: "bad state" });
    send(target, { type: "cyhdev:eu5-state", v: 2, state: "a".repeat(2001) });
    send(target, { type: "cyhdev:eu5-state", v: 2, state: "s1~q.x" }, "https://cyhdev.com");
    expect(router.navigate).not.toHaveBeenCalled();
  });

  it("sends a second message after the root class changes, and only when settings differ", async () => {
    const { target, postMessage } = mountPage();
    // Before the first ready nothing is sent, even when the root changes.
    document.documentElement.setAttribute("data-accent", "teal");
    await Promise.resolve();
    expect(postMessage).not.toHaveBeenCalled();

    send(target, { type: "cyhdev:eu5-ready", v: 2 });
    expect(postMessage).toHaveBeenCalledOnce();

    // An unrelated attribute change leaves the settings equal, so nothing is sent.
    document.documentElement.setAttribute("data-accent", "amber");
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(postMessage).toHaveBeenCalledOnce();

    setTheme("dark");
    document.documentElement.classList.add("dark");
    await waitFor(() => expect(postMessage).toHaveBeenCalledTimes(2));
    const second = postMessage.mock.calls[1]?.[0] as { theme: { mode: string }; viewState?: string };
    expect(second.theme.mode).toBe("dark");
    expect("viewState" in second).toBe(false);
    document.documentElement.removeAttribute("data-accent");
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