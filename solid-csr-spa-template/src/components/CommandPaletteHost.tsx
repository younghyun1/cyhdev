import { Loading, Show, lazy, onSettled } from "solid-js";
import type { JSX } from "@solidjs/web";
import {
  closeCommandPalette,
  commandPaletteOpen,
  isPaletteShortcut,
  setCommandPaletteOpen,
} from "../state/command_palette";

const CommandPalette = lazy(() => import("./CommandPalette"));

function isEditable(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    target instanceof HTMLTextAreaElement ||
    (target instanceof HTMLInputElement && target.type !== "checkbox" && target.type !== "radio")
  );
}

/**
 * Owns the Ctrl/Command+K shortcut and mounts the lazily loaded palette. The
 * shortcut is ignored inside other text fields, where editors may bind it,
 * but still closes the palette from its own input.
 */
export default function CommandPaletteHost(): JSX.Element {
  onSettled(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isPaletteShortcut(event)) return;
      if (!commandPaletteOpen() && isEditable(event.target)) return;
      event.preventDefault();
      setCommandPaletteOpen(!commandPaletteOpen());
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  });

  return (
    <Show when={commandPaletteOpen()}>
      <Loading>
        <CommandPalette onClose={closeCommandPalette} />
      </Loading>
    </Show>
  );
}
