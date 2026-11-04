import { describe, expect, it } from "vitest";
import { UI_TEXT_KEYS } from "./keys";
import { EN_US_DEFAULT_TEXTS } from "./defaults/en-us";
import { KO_KR_DEFAULT_TEXTS } from "./defaults/ko-kr";
import { isUiLocale, LOCAL_TEXT_LOADERS, localeDirection, resolveLocale, UI_LOCALES } from "./locales";
import backendEnglish from "../../../rust-be-template/i18n/ui/en-US.json";
import backendKorean from "../../../rust-be-template/i18n/ui/ko-KR.json";

const placeholders = (value: string) => [...value.matchAll(/\{[A-Za-z0-9_]+\}/g)].map(([token]) => token).sort();

describe("UI locales", () => {
  it("keeps browser defaults and backend sources identical", () => {
    expect(backendEnglish).toEqual(EN_US_DEFAULT_TEXTS);
    expect(backendKorean).toEqual(KO_KR_DEFAULT_TEXTS);
    expect(new Set(UI_TEXT_KEYS).size).toBe(UI_TEXT_KEYS.length);
  });

  for (const { tag } of UI_LOCALES) {
    it(`${tag} covers every key, placeholder, and the exact top-bar brand`, async () => {
      const texts = await LOCAL_TEXT_LOADERS[tag]();
      expect(Object.keys(texts).sort()).toEqual([...UI_TEXT_KEYS].sort());
      for (const key of UI_TEXT_KEYS) {
        expect(texts[key].trim(), key).not.toBe("");
        expect(placeholders(texts[key]), key).toEqual(placeholders(EN_US_DEFAULT_TEXTS[key]));
      }
      expect(texts["top_bar.site_title"]).toBe("Younghyun's Blog");
      if (tag !== "en-US") {
        for (const key of ["home.hero.role", "home.hero.summary", "home.hero.principle"] as const) {
          expect(texts[key], key).not.toBe(EN_US_DEFAULT_TEXTS[key]);
        }
        // Cognates and product names may match; an English placeholder catalog may not.
        const translated = UI_TEXT_KEYS.filter((key) => texts[key] !== EN_US_DEFAULT_TEXTS[key]);
        expect(translated.length).toBeGreaterThan(UI_TEXT_KEYS.length * 0.9);
      }
    });
  }

  it("marks exactly the Arabic-script languages as right to left", () => {
    expect(UI_LOCALES.filter(({ tag }) => localeDirection(tag) === "rtl").map(({ tag }) => tag))
      .toEqual(["ar", "ur-PK", "fa-IR"]);
  });

  it("negotiates regions while preserving Chinese script preferences", () => {
    for (const [input, expected] of [
      ["fr-CA", "fr-FR"], ["es-MX", "es-ES"], ["de-AT", "de-DE"],
      ["ja", "ja-JP"], ["KO_kr", "ko-KR"], ["en-GB", "en-US"],
      ["zh", "zh-Hans"], ["zh-SG", "zh-Hans"], ["zh-TW", "zh-Hant"],
      ["zh-HK", "zh-Hant"], ["zh-MO", "zh-Hant"],
      ["zh-Hans-TW", "zh-Hans"], ["zh-Hant-CN", "zh-Hant"],
      ["ar-EG", "ar"], ["hi", "hi-IN"], ["bn-IN", "bn-BD"], ["te", "te-IN"],
      ["pt-PT", "pt-BR"], ["in", "id-ID"], ["ru-UA", "ru-RU"], ["ur-IN", "ur-PK"],
      ["vi", "vi-VN"], ["fil-PH", "tl-PH"], ["tl", "tl-PH"], ["fa-AF", "fa-IR"],
      ["it-CH", "it-IT"], ["pl", "pl-PL"], ["sv-FI", "sv-SE"], ["ro-MD", "ro-RO"],
      ["hr-HR", "sh"], ["sr-Latn-RS", "sh"], ["bs", "sh"], ["cnr", "sh"], ["sh", "sh"],
    ]) {
      expect(resolveLocale(input ?? "")).toBe(expected);
    }
    expect(resolveLocale("unknown")).toBeNull();
    expect(isUiLocale(null)).toBe(false);
    expect(isUiLocale("fr-FR")).toBe(true);
    expect(isUiLocale("fr-CA")).toBe(false);
  });
});
