import type { JSX } from "@solidjs/web";
import { openCommandPalette, paletteShortcutLabel } from "../state/command_palette";
import { t } from "../state/i18n";

/** Header trigger for the command palette; the label collapses on narrow desktops. */
export default function CommandPaletteButton(): JSX.Element {
  return (
    <button
      type="button"
      class="cmdk-trigger"
      aria-label={t("command_palette.open")}
      aria-keyshortcuts="Control+K Meta+K"
      onClick={openCommandPalette}
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="h-4 w-4"
        aria-hidden="true"
      >
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" />
      </svg>
      <span class="cmdk-trigger-label">{t("command_palette.button")}</span>
      <kbd class="cmdk-trigger-kbd" aria-hidden="true">
        {paletteShortcutLabel()}
      </kbd>
    </button>
  );
}
