import { describe, expect, it } from "vitest";

import { isTypingTarget, shortcutFor, type ShortcutEvent } from "./shortcuts";

const press = (key: string, extra: Partial<ShortcutEvent> = {}): ShortcutEvent => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  defaultPrevented: false,
  target: document.body,
  ...extra,
});

describe("shortcutFor", () => {
  it("maps the four archive keys", () => {
    expect(shortcutFor(press("/"), false)).toBe("focus-search");
    expect(shortcutFor(press("j"), false)).toBe("next");
    expect(shortcutFor(press("k"), false)).toBe("previous");
    expect(shortcutFor(press("t"), false)).toBe("toggle-translation");
    expect(shortcutFor(press("x"), false)).toBeUndefined();
    expect(shortcutFor(press("toString"), false)).toBeUndefined();
  });

  it("leaves the site's own modified shortcuts alone", () => {
    expect(shortcutFor(press("k", { ctrlKey: true }), false)).toBeUndefined();
    expect(shortcutFor(press("k", { metaKey: true }), false)).toBeUndefined();
    expect(shortcutFor(press("t", { altKey: true }), false)).toBeUndefined();
  });

  it("does not act while typing, in a dialog, or after another handler took the key", () => {
    for (const tag of ["input", "textarea", "select"]) {
      expect(shortcutFor(press("j", { target: document.createElement(tag) }), false)).toBeUndefined();
    }
    const editable = document.createElement("div");
    editable.contentEditable = "true";
    Object.defineProperty(editable, "isContentEditable", { value: true });
    expect(shortcutFor(press("j", { target: editable }), false)).toBeUndefined();
    expect(shortcutFor(press("j"), true)).toBeUndefined();
    expect(shortcutFor(press("j", { defaultPrevented: true }), false)).toBeUndefined();
  });

  it("still acts on buttons and non-text inputs", () => {
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    expect(isTypingTarget(checkbox)).toBe(false);
    expect(isTypingTarget(document.createElement("button"))).toBe(false);
    expect(isTypingTarget(null)).toBe(false);
    const date = document.createElement("input");
    date.type = "date";
    expect(isTypingTarget(date)).toBe(true);
  });
});
