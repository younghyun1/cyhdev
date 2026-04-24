import { createSignal } from "solid-js";
import type { UiLocale, UiTextKey } from "../i18n/keys";
import { UI_TEXT_KEYS } from "../i18n/keys";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
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

function normalizeTexts(rawTexts: Record<string, string>): Record<UiTextKey, string> {
  const next: Record<UiTextKey, string> = { ...EN_US_DEFAULT_TEXTS };
  for (const key of UI_TEXT_KEYS) {
    const value = rawTexts[key];
    if (typeof value === "string" && value.length > 0) {
      next[key] = value;
    }
  }
  return next;
}

export const [locale, setLocaleSignal] = createSignal<UiLocale>(initialLocale());
export const [texts, setTexts] =
  createSignal<Record<UiTextKey, string>>(EN_US_DEFAULT_TEXTS);

export function applyLocale(nextLocale: UiLocale) {
  if (typeof document === "undefined") return;
  document.documentElement.lang = nextLocale === "ko-KR" ? "ko" : "en";
}

export async function loadUiTextBundle(nextLocale = locale()) {
  try {
    const response = await i18nApi.getUiTextBundle(nextLocale);
    if (response.success && response.data?.texts) {
      setTexts(normalizeTexts(response.data.texts));
      return;
    }
  } catch {
    setTexts(EN_US_DEFAULT_TEXTS);
  }
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
  return texts()[key] ?? EN_US_DEFAULT_TEXTS[key];
}
