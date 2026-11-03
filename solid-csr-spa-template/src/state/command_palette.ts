import { createSignal } from "solid-js";

/**
 * Open state for the site-wide command palette. The palette module itself is
 * a lazy chunk mounted by `CommandPaletteHost` only while this is true.
 */
export const [commandPaletteOpen, setCommandPaletteOpen] = createSignal(false);

export function openCommandPalette(): void {
  setCommandPaletteOpen(true);
}

export function closeCommandPalette(): void {
  setCommandPaletteOpen(false);
}

/** True for Ctrl+K, or Command+K on Apple platforms. */
export function isPaletteShortcut(event: KeyboardEvent): boolean {
  return (
    event.key.toLowerCase() === "k" &&
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey
  );
}

/** Label for the shortcut hint; Apple platforms use the Command symbol. */
export function paletteShortcutLabel(): string {
  const platform =
    (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData
      ?.platform ?? navigator.platform;
  return /mac|iphone|ipad/i.test(platform) ? "⌘K" : "Ctrl K";
}
