import { createSignal, flush } from "solid-js";

/**
 * Selectable decorative accent. Amber is the default and is represented by
 * the absence of `data-accent`; the CSS values for each preset live in
 * `index.css`. "custom" is the visitor's own color, set through
 * `state/custom_accent`, which stays out of the initial chunk. The pre-paint
 * script in `index.html` mirrors `readAccent`.
 */
export const ACCENTS = ["amber", "lagoon", "cobalt", "orchid", "graphite"] as const;
export type AccentPreset = (typeof ACCENTS)[number];
export type Accent = AccentPreset | "custom";

const STORAGE_KEY = "accent";
const CUSTOM_STORAGE_KEY = "accent-custom";
const STORED_CUSTOM_PATTERN = /^(?:--custom-accent[a-z-]*:#[0-9a-f]{6};){7}$/u;
const STORED_SOURCE_PATTERN = /--custom-accent-source:(#[0-9a-f]{6});/u;

export function isAccentPreset(value: unknown): value is AccentPreset {
  return typeof value === "string" && (ACCENTS as ReadonlyArray<string>).includes(value);
}

/** The color a stored value was made from, or null if it is not one of ours. */
export function storedCustomAccentSource(stored: string | null): string | null {
  if (!stored || !STORED_CUSTOM_PATTERN.test(stored)) return null;
  return STORED_SOURCE_PATTERN.exec(stored)?.[1] ?? null;
}

function readStored(key: string): string | null {
  try {
    return typeof window !== "undefined" ? window.localStorage.getItem(key) : null;
  } catch {
    return null;
  }
}

function writeStored(key: string, value: string | null): void {
  try {
    if (value === null) window.localStorage.removeItem(key);
    else window.localStorage.setItem(key, value);
  } catch {
    // Storage may be unavailable; the choice then lasts for this page only.
  }
}

function readCustomColor(): string | null {
  return storedCustomAccentSource(readStored(CUSTOM_STORAGE_KEY));
}

function readAccent(): Accent {
  const stored = readStored(STORAGE_KEY);
  if (stored === "custom") return readCustomColor() ? "custom" : "amber";
  return isAccentPreset(stored) ? stored : "amber";
}

export const [accent, setAccentSignal] = createSignal<Accent>(readAccent());
/** The last custom color, kept while a preset is active so it can be reselected. */
export const [customColor, setCustomColorSignal] = createSignal<string | null>(readCustomColor());

export function setAccent(next: AccentPreset): void {
  setAccentSignal(next);
  flush();
  const root = document.documentElement;
  if (next === "amber") root.removeAttribute("data-accent");
  else root.setAttribute("data-accent", next);
  writeStored(STORAGE_KEY, next === "amber" ? null : next);
}

/** Records a custom color whose declarations `state/custom_accent` already applied. */
export function commitCustomAccent(color: string, declarations: string): void {
  setCustomColorSignal(color);
  setAccentSignal("custom");
  flush();
  document.documentElement.setAttribute("data-accent", "custom");
  writeStored(STORAGE_KEY, "custom");
  writeStored(CUSTOM_STORAGE_KEY, declarations);
}
