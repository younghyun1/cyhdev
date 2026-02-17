import { createSignal } from "solid-js";

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
}

export function toggleTheme() {
  const next = theme() === "dark" ? "light" : "dark";
  setTheme(next);
  localStorage.setItem("theme", next);
}
