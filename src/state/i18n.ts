import { createSignal } from "solid-js";
import type { UiLocale, UiTextKey } from "../i18n/keys";
import { UI_TEXT_KEYS } from "../i18n/keys";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { KO_KR_DEFAULT_TEXTS } from "../i18n/defaults/ko-kr";
import { i18nApi } from "../services/all_api";

const UI_LOCALE_STORAGE_KEY = "ui_locale";

function isUiLocale(value: string | null): value is UiLocale {
  return value === "en-US" || value === "ko-KR";
}

function browserDefaultLocale(): UiLocale {
  if (typeof navigator === "undefined") return "en-US";
  const language = navigator.language.toLowerCase();
  return language.startsWith("ko") ? "ko-KR" : "en-US";
}

function initialLocale(): UiLocale {
  if (typeof window === "undefined") return "en-US";
  const persisted = localStorage.getItem(UI_LOCALE_STORAGE_KEY);
  return isUiLocale(persisted) ? persisted : browserDefaultLocale();
}

function defaultTextsForLocale(nextLocale: UiLocale): Record<UiTextKey, string> {
  return nextLocale === "ko-KR" ? KO_KR_DEFAULT_TEXTS : EN_US_DEFAULT_TEXTS;
}

function normalizeTexts(
  nextLocale: UiLocale,
  rawTexts: Record<string, string>,
): Record<UiTextKey, string> {
  const next: Record<UiTextKey, string> = {
    ...defaultTextsForLocale(nextLocale),
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
  createSignal<Record<UiTextKey, string>>(defaultTextsForLocale(INITIAL_LOCALE));

export function applyLocale(nextLocale: UiLocale) {
  if (typeof document === "undefined") return;
  document.documentElement.lang = nextLocale === "ko-KR" ? "ko" : "en";
}

export async function loadUiTextBundle(nextLocale = locale()) {
  try {
    const response = await i18nApi.getUiTextBundle(nextLocale);
    if (response.success && response.data?.texts) {
      setTexts(normalizeTexts(nextLocale, response.data.texts));
      return;
    }
  } catch {
    // Keep the app renderable with the typed local default bundle.
  }
  setTexts(defaultTextsForLocale(nextLocale));
}

export async function setLocale(nextLocale: UiLocale) {
  setLocaleSignal(nextLocale);
  if (typeof window !== "undefined") {
    localStorage.setItem(UI_LOCALE_STORAGE_KEY, nextLocale);
  }
  applyLocale(nextLocale);
  await loadUiTextBundle(nextLocale);
}

export function t(key: UiTextKey): string {
  return texts()[key] ?? defaultTextsForLocale(locale())[key];
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
