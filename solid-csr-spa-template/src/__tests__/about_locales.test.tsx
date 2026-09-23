import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import About from "../pages/about";
import AboutBlog from "../pages/about_blog";
import { setLocaleSignal } from "../state/i18n";
import type { UiLocale } from "../i18n/keys";

const headings: ReadonlyArray<{
  locale: UiLocale;
  about: string;
  blog: string;
}> = [
  { locale: "en-US", about: "About", blog: "Blog Tech Stack" },
  { locale: "ko-KR", about: "소개", blog: "블로그 기술 구성" },
  { locale: "fr-FR", about: "À propos", blog: "Technologies du blog" },
  { locale: "es-ES", about: "Sobre mí", blog: "Tecnologías del blog" },
  { locale: "zh-Hans", about: "关于我", blog: "博客技术栈" },
  { locale: "zh-Hant", about: "關於我", blog: "部落格技術架構" },
  { locale: "ja-JP", about: "プロフィール", blog: "ブログの技術構成" },
  { locale: "de-DE", about: "Über mich", blog: "Technologie-Stack des Blogs" },
];

afterEach(() => {
  cleanup();
  setLocaleSignal("en-US");
});

describe("About page localization", () => {
  it.each([
    { Page: About, heading: "about" as const },
    { Page: AboutBlog, heading: "blog" as const },
  ])("updates $heading content when the language changes", async ({ Page, heading }) => {
    setLocaleSignal("en-US");
    render(() => <Page />);

    for (const entry of headings) {
      setLocaleSignal(entry.locale);
      await waitFor(() => {
        const main = screen.getByRole("main");
        expect(main.getAttribute("lang")).toBe(entry.locale);
        expect(screen.getByRole("heading", { level: 1 }).textContent?.trim()).toBe(entry[heading]);
      });
    }
  });
});
