import { createSignal, flush } from "solid-js";

/**
 * Selectable decorative accent. Amber is the default and is represented by
 * the absence of `data-accent`; the CSS values for each preset live in
 * `index.css`. The pre-paint script in `index.html` mirrors `readAccent`.
 */
export const ACCENTS = ["amber", "lagoon", "cobalt", "orchid", "graphite"] as const;
export type Accent = (typeof ACCENTS)[number];

const STORAGE_KEY = "accent";

export function isAccent(value: unknown): value is Accent {
  return typeof value === "string" && (ACCENTS as ReadonlyArray<string>).includes(value);
}

function readAccent(): Accent {
  try {
    const stored =
      typeof window !== "undefined" ? window.localStorage.getItem(STORAGE_KEY) : null;
    return isAccent(stored) ? stored : "amber";
  } catch {
    return "amber";
  }
}

function persistAccent(next: Accent): void {
  try {
    if (next === "amber") window.localStorage.removeItem(STORAGE_KEY);
    else window.localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Storage may be unavailable; the choice then lasts for this page only.
  }
}

export const [accent, setAccentSignal] = createSignal<Accent>(readAccent());

export function applyAccent(next: Accent): void {
  const root = document.documentElement;
  if (next === "amber") root.removeAttribute("data-accent");
  else root.setAttribute("data-accent", next);
}

export function setAccent(next: Accent): void {
  setAccentSignal(next);
  flush();
  applyAccent(next);
  persistAccent(next);
}

/** Next preset in a fixed cycle, used by the command palette shortcut. */
export function nextAccent(current: Accent): Accent {
  const index = ACCENTS.indexOf(current);
  return ACCENTS[(index + 1) % ACCENTS.length] ?? "amber";
}
