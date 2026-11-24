import { useLocation, useNavigate } from "@solidjs/router";
import { type Component, onSettled } from "solid-js";

import type { UiLocale } from "../i18n/keys";
import { localeDirection } from "../i18n/locales";
import { locale, t } from "../state/i18n";
import { motionEnabled } from "../state/motion";
import "../styles/eu5-locations-db.css";

/** Version of the parent/frame protocol; see docs/design/fe/host-protocol.md in the application repository. */
export const EU5_PROTOCOL_VERSION = 2;
export const EU5_HOST_MESSAGE_TYPE = "cyhdev:eu5-host";
export const EU5_READY_MESSAGE_TYPE = "cyhdev:eu5-ready";
export const EU5_STATE_MESSAGE_TYPE = "cyhdev:eu5-state";
export const EU5_VIEW_STATE_PARAM = "view";
export const EU5_VIEW_STATE_MAX_LENGTH = 2000;
export const EU5_DEFAULT_VIEW_STATE = "s1";
export const EU5_VIEW_STATE_PATTERN = /^[A-Za-z0-9_.~*!-]{1,2000}$/;

export const EU5_PALETTE_TOKENS = [
  "paper",
  "surface",
  "ink",
  "inkMuted",
  "inkFaint",
  "line",
  "lineStrong",
  "accent",
  "accentStrong",
  "accentSoft",
  "ok",
  "danger",
] as const;
export type Eu5PaletteToken = (typeof EU5_PALETTE_TOKENS)[number];
export type Eu5ThemeMode = "light" | "dark";
export type Eu5Palette = Readonly<Record<Eu5PaletteToken, string>>;
export type Eu5HostTheme = Readonly<{ mode: Eu5ThemeMode }> & Eu5Palette;

/** Site custom property for each token, such as inkMuted to --ink-muted. */
export const EU5_PALETTE_PROPERTIES: Readonly<
  Record<Eu5PaletteToken, `--${string}`>
> = {
  paper: "--paper",
  surface: "--surface",
  ink: "--ink",
  inkMuted: "--ink-muted",
  inkFaint: "--ink-faint",
  line: "--line",
  lineStrong: "--line-strong",
  accent: "--accent",
  accentStrong: "--accent-strong",
  accentSoft: "--accent-soft",
  ok: "--ok",
  danger: "--danger",
};

/** The amber values from index.css, used for any token that cannot be resolved. */
export const EU5_DEFAULT_PALETTES: Readonly<
  Record<Eu5ThemeMode, Eu5Palette>
> = {
  light: {
    paper: "#f6f1e8",
    surface: "#fdfaf4",
    ink: "#211d18",
    inkMuted: "#6e655a",
    inkFaint: "#9a9084",
    line: "#e3dacb",
    lineStrong: "#c9bfae",
    accent: "#b45309",
    accentStrong: "#92400e",
    accentSoft: "#f6e5c4",
    ok: "#047857",
    danger: "#be123c",
  },
  dark: {
    paper: "#000000",
    surface: "#141210",
    ink: "#f0ece5",
    inkMuted: "#a39a8f",
    inkFaint: "#6f675d",
    line: "#262220",
    lineStrong: "#3d3731",
    accent: "#fbbf24",
    accentStrong: "#fcd34d",
    accentSoft: "#422006",
    ok: "#34d399",
    danger: "#fb7185",
  },
};

export type Eu5HostMessage = Readonly<{
  type: typeof EU5_HOST_MESSAGE_TYPE;
  v: typeof EU5_PROTOCOL_VERSION;
  theme: Eu5HostTheme;
  locale: UiLocale;
  dir: "ltr" | "rtl";
  reducedMotion: boolean;
  viewState?: string;
}>;
export type Eu5MessageEvent = Pick<
  MessageEvent<unknown>,
  "source" | "origin" | "data"
>;

const NUMBER_PATTERN = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?$/;

const toHexByte = (value: number): string =>
  value.toString(16).padStart(2, "0");

const toHexColor = (r: number, g: number, b: number): string =>
  `#${toHexByte(r)}${toHexByte(g)}${toHexByte(b)}`;

/** Parses one CSS number or percentage token; null when malformed. */
const parseCssNumber = (
  token: string,
): Readonly<{ value: number; percent: boolean }> | null => {
  const percent = token.endsWith("%");
  const body = percent ? token.slice(0, -1) : token;
  if (!NUMBER_PATTERN.test(body)) return null;
  const value = Number(body);
  return Number.isFinite(value) ? { value, percent } : null;
};

/** True when the optional alpha token is exactly 1 or 100%. */
const isOpaqueAlpha = (token: string | undefined): boolean => {
  if (token === undefined) return true;
  const alpha = parseCssNumber(token);
  if (alpha === null) return false;
  return alpha.percent ? alpha.value === 100 : alpha.value === 1;
};

/**
 * Splits function arguments into channel tokens and an optional alpha token,
 * accepting both the comma form and the space-and-slash form.
 */
const splitColorArguments = (
  inner: string,
): Readonly<{ channels: string[]; alpha: string | undefined }> | null => {
  const separator = (text: string): string[] =>
    text.trim().split(/[\s,]+/).filter((part) => part.length > 0);
  const slash = inner.split("/");
  if (slash.length > 2) return null;
  const main = separator(slash[0] ?? "");
  if (slash.length === 2) {
    const alpha = separator(slash[1] ?? "");
    return alpha.length === 1 && main.length === 3
      ? { channels: main, alpha: alpha[0] }
      : null;
  }
  if (main.length === 3) return { channels: main, alpha: undefined };
  if (main.length === 4) return { channels: main.slice(0, 3), alpha: main[3] };
  return null;
};

const scaleChannels = (
  tokens: readonly string[],
  maximum: number,
): [number, number, number] | null => {
  const out: number[] = [];
  for (const token of tokens) {
    const parsed = parseCssNumber(token);
    if (parsed === null || parsed.percent) return null;
    if (parsed.value < 0 || parsed.value > maximum) return null;
    out.push(Math.round((parsed.value / maximum) * 255));
  }
  const [r, g, b] = out;
  return r === undefined || g === undefined || b === undefined
    ? null
    : [r, g, b];
};

/** #rrggbb from hex, rgb(), rgba() or color(srgb ...) text with alpha 1; otherwise null. */
export const normalizeCssColor = (value: string): string | null => {
  const text = value.trim().toLowerCase();
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/.exec(text);
  if (hex?.[1] !== undefined) {
    const digits = hex[1];
    if (digits.length === 3) {
      return `#${[...digits].map((digit) => digit + digit).join("")}`;
    }
    if (digits.length === 8) {
      return digits.endsWith("ff") ? `#${digits.slice(0, 6)}` : null;
    }
    return `#${digits}`;
  }
  const rgb = /^rgba?\(([^()]*)\)$/.exec(text);
  if (rgb?.[1] !== undefined) {
    const args = splitColorArguments(rgb[1]);
    if (args === null || !isOpaqueAlpha(args.alpha)) return null;
    const channels = scaleChannels(args.channels, 255);
    return channels === null ? null : toHexColor(...channels);
  }
  const srgb = /^color\(\s*srgb\s+([^()]*)\)$/.exec(text);
  if (srgb?.[1] !== undefined) {
    const args = splitColorArguments(srgb[1]);
    if (args === null || !isOpaqueAlpha(args.alpha)) return null;
    const channels = scaleChannels(args.channels, 1);
    return channels === null ? null : toHexColor(...channels);
  }
  return null;
};

/** All 12 tokens through readColor and normalizeCssColor, defaulting per token. */
export const resolveEu5Theme = (
  mode: Eu5ThemeMode,
  readColor: (property: string) => string | null,
): Eu5HostTheme => {
  const defaults = EU5_DEFAULT_PALETTES[mode];
  const palette: Record<Eu5PaletteToken, string> = { ...defaults };
  for (const token of EU5_PALETTE_TOKENS) {
    const raw = readColor(EU5_PALETTE_PROPERTIES[token]);
    const normalized = raw === null ? null : normalizeCssColor(raw);
    palette[token] = normalized ?? defaults[token];
  }
  return { mode, ...palette };
};

/** Paints one pixel with the colour text and reads it back as sRGB; null when it is not opaque. */
const paintToHex = (
  context: CanvasRenderingContext2D,
  text: string,
): string | null => {
  context.clearRect(0, 0, 1, 1);
  // An unparsable fillStyle is ignored by the browser, so start from a known colour.
  context.fillStyle = "#000000";
  context.fillStyle = text;
  context.fillRect(0, 0, 1, 1);
  const data = context.getImageData(0, 0, 1, 1).data;
  const [r, g, b, a] = [data[0], data[1], data[2], data[3]];
  if (r === undefined || g === undefined || b === undefined || a !== 255) {
    return null;
  }
  return toHexColor(r, g, b);
};

/** DOM adapter: probe element, computed style, canvas fallback. Not pure. */
export const createEu5ColorReader = (
  doc: Document,
): ((property: string) => string | null) => {
  let canvasContext: CanvasRenderingContext2D | null | undefined;
  const context = (): CanvasRenderingContext2D | null => {
    if (canvasContext === undefined) {
      try {
        canvasContext = doc
          .createElement("canvas")
          .getContext("2d", { willReadFrequently: true });
      } catch {
        canvasContext = null;
      }
    }
    return canvasContext;
  };
  return (property) => {
    const view = doc.defaultView;
    if (!view || !doc.body) return null;
    // An undefined property would silently inherit the body colour, so check it exists.
    if (view.getComputedStyle(doc.documentElement).getPropertyValue(property).trim() === "") {
      return null;
    }
    const probe = doc.createElement("div");
    probe.style.display = "none";
    probe.style.color = `var(${property})`;
    doc.body.appendChild(probe);
    try {
      const computed = view.getComputedStyle(probe).color;
      const direct = normalizeCssColor(computed);
      if (direct !== null) return direct;
      const canvas = context();
      if (canvas === null || computed === "") return null;
      return paintToHex(canvas, computed);
    } catch {
      return null;
    } finally {
      probe.remove();
    }
  };
};

export const isEu5ViewState = (value: unknown): value is string =>
  typeof value === "string" && EU5_VIEW_STATE_PATTERN.test(value);

/** Derives dir with localeDirection; includes viewState only when it passes isEu5ViewState. */
export const buildEu5HostMessage = (
  input: Readonly<{
    theme: Eu5HostTheme;
    locale: UiLocale;
    reducedMotion: boolean;
    viewState?: string | null;
  }>,
): Eu5HostMessage => {
  const base = {
    type: EU5_HOST_MESSAGE_TYPE,
    v: EU5_PROTOCOL_VERSION,
    theme: input.theme,
    locale: input.locale,
    dir: localeDirection(input.locale),
    reducedMotion: input.reducedMotion,
  } as const;
  return isEu5ViewState(input.viewState)
    ? { ...base, viewState: input.viewState }
    : base;
};

/** True when theme, locale, direction and motion are equal; viewState is not compared. */
export const sameEu5HostSettings = (
  a: Eu5HostMessage,
  b: Eu5HostMessage,
): boolean =>
  a.theme.mode === b.theme.mode &&
  EU5_PALETTE_TOKENS.every((token) => a.theme[token] === b.theme[token]) &&
  a.locale === b.locale &&
  a.dir === b.dir &&
  a.reducedMotion === b.reducedMotion;

/**
 * Returns the message body when the event comes from this page's own frame
 * with the given type and version. The frame is sandboxed without
 * `allow-same-origin`, so its messages carry the opaque origin `"null"`;
 * identity comes from the source window, and the origin check rejects a
 * same-origin sender that happens to hold the window reference.
 */
const readFrameMessage = (
  event: Eu5MessageEvent,
  frameWindow: MessageEventSource | null,
  type: string,
): Readonly<Record<string, unknown>> | null => {
  if (frameWindow === null || event.source !== frameWindow) return null;
  if (event.origin !== "null") return null;
  const data: unknown = event.data;
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    return null;
  }
  const record = data as Readonly<Record<string, unknown>>;
  return record.type === type && record.v === EU5_PROTOCOL_VERSION
    ? record
    : null;
};

export const isEu5Ready = (
  event: Eu5MessageEvent,
  frameWindow: MessageEventSource | null,
): boolean =>
  readFrameMessage(event, frameWindow, EU5_READY_MESSAGE_TYPE) !== null;

/** The validated state of a state message from the frame, or null. */
export const readEu5StateMessage = (
  event: Eu5MessageEvent,
  frameWindow: MessageEventSource | null,
): string | null => {
  const body = readFrameMessage(event, frameWindow, EU5_STATE_MESSAGE_TYPE);
  const state = body?.state;
  return isEu5ViewState(state) ? state : null;
};

/** The validated view parameter of a location.search string, or null. */
export const readEu5ViewStateParam = (search: string): string | null => {
  const value = new URLSearchParams(search).get(EU5_VIEW_STATE_PARAM);
  return isEu5ViewState(value) ? value : null;
};

/**
 * search with view removed, then appended unencoded unless state is null or
 * the default. Returns "" or a string starting with "?".
 */
export const withEu5ViewStateParam = (
  search: string,
  state: string | null,
): string => {
  const params = new URLSearchParams(search);
  params.delete(EU5_VIEW_STATE_PARAM);
  const parts = [params.toString()];
  if (isEu5ViewState(state) && state !== EU5_DEFAULT_VIEW_STATE) {
    // The pattern needs no percent-encoding, and raw text keeps links readable.
    parts.push(`${EU5_VIEW_STATE_PARAM}=${state}`);
  }
  const joined = parts.filter((part) => part.length > 0).join("&");
  return joined.length > 0 ? `?${joined}` : "";
};

export const calculateViewportOffsets = (
  topBarBottom: number,
  bottomBarTop: number,
  viewportHeight: number,
): Readonly<{ top: number; bottom: number }> => ({
  top: Math.max(0, Math.ceil(topBarBottom)),
  bottom: Math.max(0, Math.ceil(viewportHeight - bottomBarTop)),
});

const OBSERVED_ROOT_ATTRIBUTES = [
  "class",
  "data-accent",
  "style",
  "lang",
  "dir",
  "data-motion",
];

const Eu5LocationsDb: Component = () => {
  let page: HTMLElement | undefined;
  let frame: HTMLIFrameElement | undefined;
  let frameReady = false;
  let lastSent: Eu5HostMessage | null = null;
  const location = useLocation();
  const navigate = useNavigate();

  // The mode comes from the root class so it always matches the colours read
  // in the same pass; the theme signal can lead the DOM by one effect.
  const currentSettings = (viewState: string | null): Eu5HostMessage => {
    const mode: Eu5ThemeMode = document.documentElement.classList.contains(
      "dark",
    )
      ? "dark"
      : "light";
    return buildEu5HostMessage({
      theme: resolveEu5Theme(mode, createEu5ColorReader(document)),
      locale: locale(),
      reducedMotion: !motionEnabled(),
      viewState,
    });
  };

  // The app runs in an opaque-origin sandbox, so no target origin can be named;
  // the payload is public presentation settings and the view string only.
  const post = (message: Eu5HostMessage): void => {
    const target = frame?.contentWindow;
    if (!target) return;
    lastSent = message;
    target.postMessage(message, "*");
  };

  let flushScheduled = false;
  const scheduleSettingsSend = (): void => {
    if (flushScheduled) return;
    flushScheduled = true;
    queueMicrotask(() => {
      flushScheduled = false;
      if (!frameReady) return;
      const next = currentSettings(null);
      if (lastSent !== null && sameEu5HostSettings(lastSent, next)) return;
      post(next);
    });
  };

  const handleMessage = (event: MessageEvent<unknown>): void => {
    const frameWindow = frame?.contentWindow ?? null;
    if (isEu5Ready(event, frameWindow)) {
      // A reload sends a new ready; answer each with a full message.
      frameReady = true;
      lastSent = null;
      post(currentSettings(readEu5ViewStateParam(location.search)));
      return;
    }
    const state = readEu5StateMessage(event, frameWindow);
    if (state === null) return;
    const next = withEu5ViewStateParam(location.search, state);
    if (next === location.search) return;
    // Replace, never push: Back leaves the page instead of stepping through filters.
    navigate(`${location.pathname}${next}${location.hash}`, {
      replace: true,
      scroll: false,
    });
  };

  onSettled(() => {
    const topBar = document.querySelector<HTMLElement>('[data-site-bar="top"]');
    const bottomBar = document.querySelector<HTMLElement>(
      '[data-site-bar="bottom"]',
    );
    window.addEventListener("message", handleMessage);
    const rootObserver =
      typeof MutationObserver === "undefined"
        ? null
        : new MutationObserver(scheduleSettingsSend);
    rootObserver?.observe(document.documentElement, {
      attributes: true,
      attributeFilter: OBSERVED_ROOT_ATTRIBUTES,
    });
    const stopHost = () => {
      rootObserver?.disconnect();
      window.removeEventListener("message", handleMessage);
    };
    if (!page || !topBar || !bottomBar) return stopHost;

    const updateOffsets = () => {
      if (!page) return;
      const offsets = calculateViewportOffsets(
        topBar.getBoundingClientRect().bottom,
        bottomBar.getBoundingClientRect().top,
        window.innerHeight,
      );
      page.style.setProperty("--eu5-top-bar-offset", `${offsets.top}px`);
      page.style.setProperty(
        "--eu5-bottom-bar-offset",
        `${offsets.bottom}px`,
      );
    };
    updateOffsets();

    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(updateOffsets);
    observer?.observe(topBar);
    observer?.observe(bottomBar);
    window.addEventListener("resize", updateOffsets);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", updateOffsets);
      stopHost();
    };
  });

  return (
    <section
      ref={(element) => {
        page = element;
      }}
      class="eu5-locations-db-page"
    >
      <iframe
        ref={(element) => {
          frame = element;
        }}
        class="eu5-locations-db-frame"
        src="/eu5-locations-db/app/index.html"
        title={t("top_bar.nav.eu5_locations_db")}
        loading="eager"
        sandbox="allow-scripts allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation"
      />
    </section>
  );
};

export default Eu5LocationsDb;
