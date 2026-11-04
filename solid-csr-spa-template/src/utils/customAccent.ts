/**
 * A visitor's own accent color. The site sets links, labels, and status text in
 * the accent, so a color that would be unreadable on the page backgrounds is
 * moved toward black or white only as far as each mode needs. A color that
 * already reads well is used exactly as entered.
 */

/** `--paper` and `--surface` in index.css, per mode. */
const BACKGROUNDS = {
  light: { paper: "#f6f1e8", surface: "#fdfaf4" },
  dark: { paper: "#000000", surface: "#141210" },
} as const;

const MIN_CONTRAST = 4.5;
/** Each step moves 2.5% toward black or white, which always clears the floor. */
const ADJUST_STEPS = 40;
const STRONG_MIX = 0.22;
const SOFT_MIX = { light: 0.14, dark: 0.2 } as const;

const HEX_PATTERN = /^#?([0-9a-f]{6})$/iu;
const SHORT_HEX_PATTERN = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/iu;
const TRIPLET_PATTERN = /^(?:rgb\s*\(\s*)?(\d{1,3})[\s,]+(\d{1,3})[\s,]+(\d{1,3})\s*\)?$/iu;

export type AccentMode = keyof typeof BACKGROUNDS;

export interface AccentTokens {
  readonly accent: string;
  readonly strong: string;
  readonly soft: string;
}

export type CustomAccent = Readonly<Record<AccentMode, AccentTokens>>;

function channels(hex: string): [number, number, number] {
  return [
    Number.parseInt(hex.slice(1, 3), 16),
    Number.parseInt(hex.slice(3, 5), 16),
    Number.parseInt(hex.slice(5, 7), 16),
  ];
}

function toHex(red: number, green: number, blue: number): string {
  const part = (value: number) => Math.round(value).toString(16).padStart(2, "0");
  return `#${part(red)}${part(green)}${part(blue)}`;
}

/** `#rrggbb` from `255, 120, 40`, `255 120 40`, `rgb(255 120 40)`, `#ff7828`, `ff7828`, or `#f72`. */
export function parseRgbInput(text: string): string | null {
  const value = text.trim();
  const hex = HEX_PATTERN.exec(value)?.[1];
  if (hex) return `#${hex.toLowerCase()}`;
  const short = SHORT_HEX_PATTERN.exec(value);
  if (short) {
    return `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`.toLowerCase();
  }
  const triplet = TRIPLET_PATTERN.exec(value);
  if (!triplet) return null;
  const [red, green, blue] = [Number(triplet[1]), Number(triplet[2]), Number(triplet[3])];
  if (red > 255 || green > 255 || blue > 255) return null;
  return toHex(red, green, blue);
}

/** `#ff7828` as `255, 120, 40`. */
export function formatRgb(hex: string): string {
  return channels(hex).join(", ");
}

function mix(from: string, to: string, amount: number): string {
  const [r1, g1, b1] = channels(from);
  const [r2, g2, b2] = channels(to);
  return toHex(
    r1 + (r2 - r1) * amount,
    g1 + (g2 - g1) * amount,
    b1 + (b2 - b1) * amount,
  );
}

function luminance(hex: string): number {
  const [red, green, blue] = channels(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

export function contrastRatio(first: string, second: string): number {
  const a = luminance(first);
  const b = luminance(second);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

function deriveMode(source: string, mode: AccentMode): AccentTokens {
  const { paper, surface } = BACKGROUNDS[mode];
  const target = mode === "light" ? "#000000" : "#ffffff";
  const softBase = mode === "light" ? surface : paper;
  let tokens: AccentTokens = { accent: source, strong: source, soft: source };
  for (let step = 0; step <= ADJUST_STEPS; step += 1) {
    const accent = mix(source, target, step / ADJUST_STEPS);
    const soft = mix(softBase, accent, SOFT_MIX[mode]);
    tokens = { accent, strong: mix(accent, target, STRONG_MIX), soft };
    if ([paper, surface, soft].every((background) => contrastRatio(accent, background) >= MIN_CONTRAST)) {
      break;
    }
  }
  return tokens;
}

export function deriveCustomAccent(source: string): CustomAccent {
  return { light: deriveMode(source, "light"), dark: deriveMode(source, "dark") };
}

/** The custom properties index.css reads under `data-accent="custom"`. */
export function customAccentProperties(source: string): ReadonlyArray<readonly [string, string]> {
  const { light, dark } = deriveCustomAccent(source);
  return [
    ["--custom-accent-source", source],
    ["--custom-accent-light", light.accent],
    ["--custom-accent-strong-light", light.strong],
    ["--custom-accent-soft-light", light.soft],
    ["--custom-accent-dark", dark.accent],
    ["--custom-accent-strong-dark", dark.strong],
    ["--custom-accent-soft-dark", dark.soft],
  ];
}

/**
 * Stored as ready-made declarations, so the pre-paint script in index.html can
 * apply them without repeating the math. It and `storedCustomAccentSource` in
 * state/accent.ts check this shape.
 */
export function serializeCustomAccent(source: string): string {
  return customAccentProperties(source)
    .map(([name, value]) => `${name}:${value};`)
    .join("");
}
