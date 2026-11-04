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
  { tag: "ar", label: "العربية" },
  { tag: "hi-IN", label: "हिन्दी" },
  { tag: "bn-BD", label: "বাংলা" },
  { tag: "te-IN", label: "తెలుగు" },
  { tag: "pt-BR", label: "Português" },
  { tag: "id-ID", label: "Bahasa Indonesia" },
  { tag: "ru-RU", label: "Русский" },
  { tag: "ur-PK", label: "اردو" },
  { tag: "vi-VN", label: "Tiếng Việt" },
  { tag: "tl-PH", label: "Tagalog" },
  { tag: "fa-IR", label: "فارسی" },
  { tag: "it-IT", label: "Italiano" },
  { tag: "pl-PL", label: "Polski" },
  { tag: "sv-SE", label: "Svenska" },
  { tag: "sh", label: "Srpskohrvatski" },
  { tag: "ro-RO", label: "Română" },
] as const satisfies readonly { tag: UiLocale; label: string }[];

const RIGHT_TO_LEFT: ReadonlySet<UiLocale> = new Set<UiLocale>(["ar", "ur-PK", "fa-IR"]);

export function localeDirection(tag: UiLocale): "ltr" | "rtl" {
  return RIGHT_TO_LEFT.has(tag) ? "rtl" : "ltr";
}

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
    case "ar": return "ar";
    case "hi": return "hi-IN";
    case "bn": return "bn-BD";
    case "te": return "te-IN";
    case "pt": return "pt-BR";
    case "id": case "in": return "id-ID";
    case "ru": return "ru-RU";
    case "ur": return "ur-PK";
    case "vi": return "vi-VN";
    case "tl": case "fil": return "tl-PH";
    case "fa": case "prs": return "fa-IR";
    case "it": return "it-IT";
    case "pl": return "pl-PL";
    case "sv": return "sv-SE";
    // One Latin-script catalog serves the mutually intelligible standards.
    case "sh": case "hbs": case "sr": case "hr": case "bs": case "cnr": return "sh";
    case "ro": return "ro-RO";
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
  "ar": async () => (await import("../../../rust-be-template/i18n/ui/ar.json")).default,
  "hi-IN": async () => (await import("../../../rust-be-template/i18n/ui/hi-IN.json")).default,
  "bn-BD": async () => (await import("../../../rust-be-template/i18n/ui/bn-BD.json")).default,
  "te-IN": async () => (await import("../../../rust-be-template/i18n/ui/te-IN.json")).default,
  "pt-BR": async () => (await import("../../../rust-be-template/i18n/ui/pt-BR.json")).default,
  "id-ID": async () => (await import("../../../rust-be-template/i18n/ui/id-ID.json")).default,
  "ru-RU": async () => (await import("../../../rust-be-template/i18n/ui/ru-RU.json")).default,
  "ur-PK": async () => (await import("../../../rust-be-template/i18n/ui/ur-PK.json")).default,
  "vi-VN": async () => (await import("../../../rust-be-template/i18n/ui/vi-VN.json")).default,
  "tl-PH": async () => (await import("../../../rust-be-template/i18n/ui/tl-PH.json")).default,
  "fa-IR": async () => (await import("../../../rust-be-template/i18n/ui/fa-IR.json")).default,
  "it-IT": async () => (await import("../../../rust-be-template/i18n/ui/it-IT.json")).default,
  "pl-PL": async () => (await import("../../../rust-be-template/i18n/ui/pl-PL.json")).default,
  "sv-SE": async () => (await import("../../../rust-be-template/i18n/ui/sv-SE.json")).default,
  "sh": async () => (await import("../../../rust-be-template/i18n/ui/sh.json")).default,
  "ro-RO": async () => (await import("../../../rust-be-template/i18n/ui/ro-RO.json")).default,
};
