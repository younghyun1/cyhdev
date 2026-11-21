import { onSettled } from "solid-js";

import { shortcutFor, type ShortcutAction } from "../../utils/kcna_archive/shortcuts";

/**
 * Listens for the archive's single-key shortcuts while the calling page is
 * mounted. `handle` returns true when it acted, which stops the browser's own
 * handling of the key.
 */
export function useArchiveShortcuts(handle: (action: ShortcutAction) => boolean): void {
  onSettled(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      const dialogOpen = document.querySelector('[role="dialog"][aria-modal="true"]') !== null;
      const action = shortcutFor(event, dialogOpen);
      if (action !== undefined && handle(action)) event.preventDefault();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });
}

/** Moves keyboard focus to the page's main text field. */
export function focusPrimaryInput(): boolean {
  const input = document.querySelector<HTMLInputElement>("[data-kcna-primary-input]");
  if (input === null) return false;
  input.focus();
  input.select();
  return true;
}
