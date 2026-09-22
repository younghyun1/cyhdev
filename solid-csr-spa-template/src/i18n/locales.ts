import type { UiLocale, UiTextKey } from "./keys";

export const UI_LOCALES = [
  { tag: "en-US", label: "English" },
  { tag: "ko-KR", label: "한국어" },
  { tag: "fr-FR", label: "Français" },
  { tag: "es-ES", label: "Español" },
  { tag: "zh-Hans", label: "简体中文" },
  { tag: "zh-Hant", label: "繁體中文" },
  { tag: "ja-JP", label: "日本語" },
  { tag: "de-DE", label: "Deutsch" },
] as const satisfies readonly { tag: UiLocale; label: string }[];

export function isUiLocale(value: string | null): value is UiLocale {
  return UI_LOCALES.some(({ tag }) => tag === value);
}

export function resolveLocale(language: string): UiLocale | null {
  const normalized = language.replaceAll("_", "-").toLowerCase();
  const [base] = normalized.split("-");
  switch (base) {
    case "en": return "en-US";
    case "ko": return "ko-KR";
    case "fr": return "fr-FR";
    case "es": return "es-ES";
    case "de": return "de-DE";
    case "ja": return "ja-JP";
    case "zh":
      if (normalized.includes("-hans")) return "zh-Hans";
      return /-(hant|tw|hk|mo)(-|$)/.test(normalized) ? "zh-Hant" : "zh-Hans";
    default: return null;
  }
}

// Each finite catalog is a separate chunk, shared with backend source synchronization.
export const LOCAL_TEXT_LOADERS: Record<UiLocale, () => Promise<Record<UiTextKey, string>>> = {
  "en-US": async () => (await import("./defaults/en-us")).EN_US_DEFAULT_TEXTS,
  "ko-KR": async () => (await import("./defaults/ko-kr")).KO_KR_DEFAULT_TEXTS,
  "fr-FR": async () => (await import("../../../rust-be-template/i18n/ui/fr-FR.json")).default,
  "es-ES": async () => (await import("../../../rust-be-template/i18n/ui/es-ES.json")).default,
  "zh-Hans": async () => (await import("../../../rust-be-template/i18n/ui/zh-Hans.json")).default,
  "zh-Hant": async () => (await import("../../../rust-be-template/i18n/ui/zh-Hant.json")).default,
  "ja-JP": async () => (await import("../../../rust-be-template/i18n/ui/ja-JP.json")).default,
  "de-DE": async () => (await import("../../../rust-be-template/i18n/ui/de-DE.json")).default,
};
