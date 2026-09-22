import { createSignal } from "solid-js";
import type { UiLocale, UiTextKey } from "../i18n/keys";
import { UI_TEXT_KEYS } from "../i18n/keys";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { isUiLocale, LOCAL_TEXT_LOADERS, resolveLocale } from "../i18n/locales";
import { i18nApi } from "../services/all_api";

const UI_LOCALE_STORAGE_KEY = "ui_locale";

function browserDefaultLocale(): UiLocale {
  if (typeof navigator === "undefined") return "en-US";
  for (const language of navigator.languages ?? [navigator.language]) {
    const matched = resolveLocale(language);
    if (matched) return matched;
  }
  return "en-US";
}

function readStoredLocale(): string | null {
  try {
    return typeof window !== "undefined"
      ? window.localStorage.getItem(UI_LOCALE_STORAGE_KEY)
      : null;
  } catch {
    return null;
  }
}

function persistLocale(nextLocale: UiLocale): void {
  try {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(UI_LOCALE_STORAGE_KEY, nextLocale);
    }
  } catch {
    // Storage may be disabled or unavailable for an opaque browser origin.
  }
}

function initialLocale(): UiLocale {
  if (typeof window === "undefined") return "en-US";
  const persisted = readStoredLocale();
  return isUiLocale(persisted) ? persisted : browserDefaultLocale();
}

function normalizeTexts(
  defaults: Record<UiTextKey, string>,
  rawTexts: Record<string, string>,
): Record<UiTextKey, string> {
  const next: Record<UiTextKey, string> = {
    ...defaults,
  };
  for (const key of UI_TEXT_KEYS) {
    const value = rawTexts[key];
    if (typeof value === "string" && value.length > 0) {
      next[key] = value;
    }
  }
  return next;
}

const INITIAL_LOCALE = initialLocale();

export const [locale, setLocaleSignal] = createSignal<UiLocale>(INITIAL_LOCALE);
export const [texts, setTexts] =
  createSignal<Record<UiTextKey, string>>(EN_US_DEFAULT_TEXTS);

let loadSequence = 0;

export function applyLocale(nextLocale: UiLocale) {
  if (typeof document === "undefined") return;
  document.documentElement.lang = nextLocale;
}

export async function loadUiTextBundle(nextLocale = locale()) {
  const sequence = ++loadSequence;
  const current = () => sequence === loadSequence && nextLocale === locale();
  let defaults = EN_US_DEFAULT_TEXTS;
  try {
    defaults = await LOCAL_TEXT_LOADERS[nextLocale]();
  } catch {
    // A stale deployment may no longer serve a lazy chunk; English stays usable.
  }
  if (!current()) return;
  setTexts(defaults);
  try {
    const response = await i18nApi.getUiTextBundle(nextLocale);
    if (!current()) return;
    if (response.success && response.data?.locale === nextLocale && response.data.texts) {
      setTexts(normalizeTexts(defaults, response.data.texts));
      return;
    }
  } catch {
    // Keep the app renderable with the typed local default bundle.
  }
}

export async function setLocale(nextLocale: UiLocale) {
  setLocaleSignal(nextLocale);
  if (typeof window !== "undefined") {
    persistLocale(nextLocale);
  }
  applyLocale(nextLocale);
  await loadUiTextBundle(nextLocale);
}

export function t(key: UiTextKey): string {
  return texts()[key] ?? EN_US_DEFAULT_TEXTS[key];
}

export function tx(
  key: UiTextKey,
  params: Record<string, string | number>,
): string {
  return t(key).replace(/\{([A-Za-z0-9_]+)\}/g, (match, name: string) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}
