import { createSignal, flush } from "solid-js";

/**
 * Site-wide animation switch. Animations are on until the visitor turns them
 * off here; the operating system's reduced-motion preference is deliberately
 * not consulted, so this is the only control. `data-motion="off"` on <html>
 * gates every stylesheet, and the pre-paint script in `index.html` mirrors
 * `readMotion` so a stored choice applies before first paint.
 */
const STORAGE_KEY = "motion";

function readMotion(): boolean {
  try {
    return (
      typeof window === "undefined" ||
      window.localStorage.getItem(STORAGE_KEY) !== "off"
    );
  } catch {
    return true;
  }
}

function persistMotion(enabled: boolean): void {
  try {
    if (enabled) window.localStorage.removeItem(STORAGE_KEY);
    else window.localStorage.setItem(STORAGE_KEY, "off");
  } catch {
    // Storage may be unavailable; the choice then lasts for this page only.
  }
}

export const [motionEnabled, setMotionSignal] = createSignal(readMotion());

export function applyMotion(enabled: boolean): void {
  const root = document.documentElement;
  if (enabled) root.removeAttribute("data-motion");
  else root.setAttribute("data-motion", "off");
}

export function setMotionEnabled(enabled: boolean): void {
  setMotionSignal(enabled);
  flush();
  applyMotion(enabled);
  persistMotion(enabled);
}

export function toggleMotion(): void {
  setMotionEnabled(!motionEnabled());
}
