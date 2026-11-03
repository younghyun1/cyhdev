import { afterEach, describe, expect, it, vi } from "vitest";
import type { UiLocale } from "../i18n/keys";
import { UI_LOCALES } from "../i18n/locales";
import {
  createLocaleContentLoader,
  type LocaleContentLoaders,
} from "../pages/about-locales/locale_content";

afterEach(() => {
  vi.restoreAllMocks();
});

function loadersWith(
  override: Partial<Record<UiLocale, () => Promise<{ default: string }>>> = {},
) {
  const calls: UiLocale[] = [];
  const loaders = {} as Record<UiLocale, LocaleContentLoaders<string>[UiLocale]>;
  for (const { tag } of UI_LOCALES) {
    loaders[tag] =
      override[tag] ??
      (() => {
        calls.push(tag);
        return Promise.resolve({ default: `content ${tag}` });
      });
  }
  return { calls, loaders };
}

describe("createLocaleContentLoader", () => {
  it("loads each locale once and reports the locale it returned", async () => {
    const { calls, loaders } = loadersWith();
    const load = createLocaleContentLoader(loaders);
    await expect(load("fr-FR")).resolves.toEqual({ locale: "fr-FR", content: "content fr-FR" });
    await load("fr-FR");
    await expect(load("ko-KR")).resolves.toEqual({ locale: "ko-KR", content: "content ko-KR" });
    expect(calls).toEqual(["fr-FR", "ko-KR"]);
  });

  it("falls back to English and labels the result as English", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const { calls, loaders } = loadersWith({
      "de-DE": () => Promise.reject(new Error("chunk missing")),
    });
    const load = createLocaleContentLoader(loaders);
    await expect(load("de-DE")).resolves.toEqual({ locale: "en-US", content: "content en-US" });
    expect(calls).toEqual(["en-US"]);
  });

  it("retries English after a failed English load", async () => {
    let attempts = 0;
    const { loaders } = loadersWith({
      "en-US": () => {
        attempts += 1;
        return attempts === 1
          ? Promise.reject(new Error("offline"))
          : Promise.resolve({ default: "content en-US" });
      },
    });
    const load = createLocaleContentLoader(loaders);
    await expect(load("en-US")).rejects.toThrow("offline");
    await expect(load("en-US")).resolves.toEqual({ locale: "en-US", content: "content en-US" });
    expect(attempts).toBe(2);
  });
});
