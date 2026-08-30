import { createSignal, flush } from "solid-js";

function getInitialTheme(): "light" | "dark" {
  if (typeof window !== "undefined") {
    const persisted = localStorage.getItem("theme");
    if (persisted === "dark" || persisted === "light") return persisted;
    if (window.matchMedia?.("(prefers-color-scheme: dark)")?.matches) {
      return "dark";
    }
  }
  return "light";
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
  localStorage.setItem("theme", next);
}
