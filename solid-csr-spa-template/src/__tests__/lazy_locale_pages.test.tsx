import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { Loading } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { UiLocale } from "../i18n/keys";
import { UI_LOCALES } from "../i18n/locales";
import {
  type LocalePageLoaders,
  createLocalePage,
} from "../pages/about-locales/lazy_locale_pages";
import { setLocaleSignal } from "../state/i18n";

afterEach(() => {
  cleanup();
  setLocaleSignal("en-US");
  vi.restoreAllMocks();
});

function loadersWith(
  override: Partial<Record<UiLocale, () => Promise<never>>> = {},
) {
  const calls: UiLocale[] = [];
  const loaders = {} as Record<UiLocale, LocalePageLoaders[UiLocale]>;
  for (const { tag } of UI_LOCALES) {
    loaders[tag] =
      override[tag] ??
      (() => {
        calls.push(tag);
        return Promise.resolve({ default: () => <p>{`page ${tag}`}</p> });
      });
  }
  return { calls, loaders };
}

describe("lazy locale pages", () => {
  it("loads only the active locale's chunk and follows language changes", async () => {
    const { calls, loaders } = loadersWith();
    const Page = createLocalePage(loaders);
    setLocaleSignal("fr-FR");
    render(() => (
      <Loading>
        <Page />
      </Loading>
    ));
    await waitFor(() => expect(screen.getByText("page fr-FR")).not.toBeNull());
    expect(calls).toEqual(["fr-FR"]);

    setLocaleSignal("ja-JP");
    await waitFor(() => expect(screen.getByText("page ja-JP")).not.toBeNull());
    expect(calls).toEqual(["fr-FR", "ja-JP"]);
  });

  it("falls back to English when a locale chunk fails to load", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const { calls, loaders } = loadersWith({
      "ko-KR": () => Promise.reject(new Error("chunk removed")),
    });
    const Page = createLocalePage(loaders);
    setLocaleSignal("ko-KR");
    render(() => (
      <Loading>
        <Page />
      </Loading>
    ));
    await waitFor(() => expect(screen.getByText("page en-US")).not.toBeNull());
    expect(calls).toEqual(["en-US"]);
  });
});
