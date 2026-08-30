import { createSignal, flush } from "solid-js";

function getInitialTheme(): "light" | "dark" {
  if (typeof window !== "undefined") {
    const persisted = readStoredTheme();
    if (persisted === "dark" || persisted === "light") return persisted;
    if (window.matchMedia?.("(prefers-color-scheme: dark)")?.matches) {
      return "dark";
    }
  }
  return "light";
}

function readStoredTheme(): string | null {
  try {
    return typeof window !== "undefined" ? window.localStorage.getItem("theme") : null;
  } catch {
    return null;
  }
}

function persistTheme(next: "light" | "dark"): void {
  try {
    if (typeof window !== "undefined") window.localStorage.setItem("theme", next);
  } catch {
    // Storage may be disabled or unavailable for an opaque browser origin.
  }
}

export const [theme, setTheme] = createSignal<"light" | "dark">(
  getInitialTheme(),
);

export function applyTheme(t: "light" | "dark") {
  const html = document.documentElement;
  html.classList.remove("light", "dark");
  html.classList.add(t);
  // Keep the browser chrome color in sync with the canvas; the same values
  // are set pre-paint by the inline script in index.html.
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) {
    meta.setAttribute("content", t === "dark" ? "#000000" : "#f6f1e8");
  }
}

export function toggleTheme() {
  const next = theme() === "dark" ? "light" : "dark";
  setTheme(next);
  // Writes land on the microtask flush; force the write through so a second
  // toggle in the same tick (or a synchronous read) sees the new value.
  flush();
  persistTheme(next);
}
