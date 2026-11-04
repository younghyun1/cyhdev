import { Show, createSignal, onCleanup } from "solid-js";
import type { JSX } from "@solidjs/web";
import { t } from "../state/i18n";
import AccentPicker from "./AccentPicker";
import MotionToggle from "./MotionToggle";

/**
 * Header popover for the accent preset and the animation switch. The trigger
 * shows the active accent so the option is visible without opening it. Outside
 * presses and Escape close the panel; Escape returns focus to the trigger.
 */
export default function AppearanceMenu(): JSX.Element {
  const [open, setOpen] = createSignal(false);
  let root: HTMLDivElement | undefined;
  let trigger: HTMLButtonElement | undefined;

  const onPointerDown = (event: PointerEvent) => {
    if (root && !root.contains(event.target as Node)) close();
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    close();
    trigger?.focus();
  };
  const removeListeners = () => {
    window.removeEventListener("pointerdown", onPointerDown);
    window.removeEventListener("keydown", onKeyDown);
  };
  const close = () => {
    setOpen(false);
    removeListeners();
  };
  const toggle = () => {
    if (open()) {
      close();
      return;
    }
    setOpen(true);
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
  };
  onCleanup(removeListeners);

  return (
    <div class="appearance-menu" ref={(element) => (root = element)}>
      <button
        ref={(element) => (trigger = element)}
        type="button"
        class="appearance-trigger"
        aria-label={t("appearance.label")}
        aria-haspopup="dialog"
        aria-expanded={open() ? "true" : "false"}
        title={t("appearance.label")}
        onClick={toggle}
      >
        <span class="appearance-trigger-dot" aria-hidden="true" />
      </button>
      <Show when={open()}>
        <div class="appearance-panel" role="dialog" aria-label={t("appearance.label")}>
          <div class="appearance-row appearance-row--stacked">
            <span class="appearance-row-label">{t("accent.label")}</span>
            <AccentPicker name="header-accent" />
          </div>
          <div class="appearance-row">
            <span class="appearance-row-label">{t("motion.label")}</span>
            <MotionToggle />
          </div>
        </div>
      </Show>
    </div>
  );
}
