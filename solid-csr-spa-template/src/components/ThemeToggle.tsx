import type { JSX } from "@solidjs/web";
import { theme, toggleThemeFrom } from "../state/theme";
import { t } from "../state/i18n";

/**
 * Sun/moon theme switch. Icons crossfade/rotate over 200ms; the class/meta
 * updates and the circular View Transition reveal happen in state/theme.ts,
 * centered on this button so keyboard activation animates from it as well.
 */
export default function ThemeToggle(): JSX.Element {
  return (
    <button
      type="button"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-sm border border-line bg-surface text-ink-muted hover:text-ink hover:bg-surface-2 transition-colors duration-90"
      aria-label={t("top_bar.aria.toggle_theme")}
      onClick={(event) => {
        const rect = event.currentTarget.getBoundingClientRect();
        toggleThemeFrom({ x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 });
      }}
    >
      <span class="relative block h-4 w-4" aria-hidden="true">
        {/* sun */}
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class={[
            "absolute inset-0 h-4 w-4 transition-all duration-200",
            theme() === "dark"
              ? "-rotate-90 scale-50 opacity-0"
              : "rotate-0 scale-100 opacity-100",
          ]}
        >
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2m0 16v2M4.93 4.93l1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
        </svg>
        {/* moon */}
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class={[
            "absolute inset-0 h-4 w-4 transition-all duration-200",
            theme() === "dark"
              ? "rotate-0 scale-100 opacity-100"
              : "rotate-90 scale-50 opacity-0",
          ]}
        >
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      </span>
    </button>
  );
}
