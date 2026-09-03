import { onCleanup, onSettled, type ParentComponent } from "solid-js";
import { Portal } from "@solidjs/web";

type MobileDialogProps = {
  readonly onClose: () => void;
  readonly overlayClass: string;
  readonly panelClass: string;
  readonly ariaLabel?: string;
  readonly ariaLabelledBy?: string;
  readonly initialFocusSelector?: string;
};

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled]):not([type='hidden'])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

/** Portal-backed modal lifecycle with focus containment and inert background. */
export const MobileDialog: ParentComponent<MobileDialogProps> = (props) => {
  let panel: HTMLDivElement | undefined;
  let dispose = () => {};

  onSettled(() => {
    const previouslyFocused =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    const appRoot = document.getElementById("app-root");
    const previousInert = appRoot?.inert ?? false;
    const previousOverflow = document.body.style.overflow;
    if (appRoot) appRoot.inert = true;
    document.body.style.overflow = "hidden";

    const focusable = () =>
      Array.from(
        panel?.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR) ?? [],
      ).filter((element) => !element.hidden);
    const requested = props.initialFocusSelector
      ? panel?.querySelector<HTMLElement>(props.initialFocusSelector)
      : null;
    queueMicrotask(() => (requested ?? focusable()[0] ?? panel)?.focus());

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        props.onClose();
        return;
      }
      if (event.key !== "Tab") return;
      const candidates = focusable();
      if (candidates.length === 0) {
        event.preventDefault();
        panel?.focus();
        return;
      }
      const first = candidates[0]!;
      const last = candidates[candidates.length - 1]!;
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown, true);

    // eslint-disable-next-line solid/reactivity -- onCleanup invokes this captured DOM teardown; it is not reactive UI.
    dispose = () => {
      document.removeEventListener("keydown", handleKeyDown, true);
      if (appRoot) appRoot.inert = previousInert;
      document.body.style.overflow = previousOverflow;
      requestAnimationFrame(() => {
        if (previouslyFocused?.isConnected) previouslyFocused.focus();
      });
    };
  });
  onCleanup(() => dispose());

  return (
    <Portal>
      <div
        class={props.overlayClass}
        onPointerDown={(event) => {
          if (event.target === event.currentTarget) props.onClose();
        }}
      >
        <div
          ref={(element) => (panel = element)}
          class={props.panelClass}
          role="dialog"
          aria-modal="true"
          aria-label={props.ariaLabel}
          aria-labelledby={props.ariaLabelledBy}
          tabindex={-1}
        >
          {props.children}
        </div>
      </div>
    </Portal>
  );
};
