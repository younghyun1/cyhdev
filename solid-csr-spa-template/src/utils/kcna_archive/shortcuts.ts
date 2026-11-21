// Keyboard shortcuts of the archive pages. The decision is a pure function of
// the event so it can be tested without a DOM.

export type ShortcutAction = "focus-search" | "next" | "previous" | "toggle-translation";

export interface ShortcutEvent {
  readonly key: string;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly altKey: boolean;
  readonly defaultPrevented: boolean;
  readonly target: EventTarget | null;
}

/** Input types that take no typed text, so single keys stay shortcuts on them. */
const NON_TEXT_INPUTS = new Set([
  "button",
  "checkbox",
  "color",
  "file",
  "image",
  "radio",
  "range",
  "reset",
  "submit",
]);

/** Whether key presses on `target` are text entry. */
export const isTypingTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement) return !NON_TEXT_INPUTS.has(target.type);
  return (
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target.isContentEditable === true
  );
};

const BY_KEY: Readonly<Record<string, ShortcutAction>> = {
  "/": "focus-search",
  j: "next",
  k: "previous",
  t: "toggle-translation",
};

/**
 * The action a key press asks for. Modified keys are the site's own (Ctrl or
 * Command with K opens its command palette), text entry keeps its keys, and a
 * modal dialog owns the keyboard while it is open.
 */
export const shortcutFor = (
  event: ShortcutEvent,
  dialogOpen: boolean,
): ShortcutAction | undefined => {
  if (event.defaultPrevented || dialogOpen) return undefined;
  if (event.ctrlKey || event.metaKey || event.altKey) return undefined;
  if (isTypingTarget(event.target)) return undefined;
  return Object.hasOwn(BY_KEY, event.key) ? BY_KEY[event.key] : undefined;
};
