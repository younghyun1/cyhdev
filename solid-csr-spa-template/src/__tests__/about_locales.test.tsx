import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

const routeParams = vi.hoisted(() => ({}) as { page?: string });
vi.mock("@solidjs/router", () => ({ useParams: () => routeParams }));

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
  { locale: "ar", about: "حول", blog: "التقنيات المستخدمة في المدونة" },
  { locale: "hi-IN", about: "परिचय", blog: "ब्लॉग का टेक स्टैक" },
  { locale: "bn-BD", about: "পরিচিতি", blog: "ব্লগের টেক স্ট্যাক" },
  { locale: "te-IN", about: "పరిచయం", blog: "బ్లాగ్ టెక్ స్టాక్" },
  { locale: "pt-BR", about: "Sobre", blog: "Stack técnica do blog" },
  { locale: "id-ID", about: "Tentang", blog: "Stack Teknologi Blog" },
  { locale: "ru-RU", about: "Обо мне", blog: "Технологический стек блога" },
  { locale: "ur-PK", about: "تعارف", blog: "بلاگ کا ٹیک اسٹیک" },
  { locale: "vi-VN", about: "Giới thiệu", blog: "Công nghệ của blog" },
  { locale: "tl-PH", about: "Tungkol", blog: "Tech Stack ng Blog" },
  { locale: "fa-IR", about: "درباره", blog: "فناوری‌های وبلاگ" },
  { locale: "it-IT", about: "Info", blog: "Stack tecnologico del blog" },
  { locale: "pl-PL", about: "O mnie", blog: "Stos technologiczny bloga" },
  { locale: "sv-SE", about: "Om", blog: "Bloggens teknikstack" },
  { locale: "sh", about: "O meni", blog: "Tehnološki stek bloga" },
  { locale: "ro-RO", about: "Despre", blog: "Stack-ul tehnic al blogului" },
];

afterEach(() => {
  cleanup();
  setLocaleSignal("en-US");
  delete routeParams.page;
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
      // Each locale page is a lazy chunk that Vitest transforms on first
      // import, which can exceed the default one-second wait on a busy machine.
      await waitFor(
        () => {
          const main = screen.getByRole("main");
          expect(main.getAttribute("lang")).toBe(entry.locale);
          expect(screen.getByRole("heading", { level: 1 }).textContent?.trim()).toBe(entry[heading]);
        },
        { timeout: 10_000 },
      );
    }
  }, 60_000);

  it.each([
    { Page: About, page: "experience", heading: "Experience", section: "Software engineering" },
    { Page: About, page: "background", heading: "Background", section: "Research" },
    { Page: AboutBlog, page: "performance", heading: "Performance", section: "Results" },
  ])("renders the $page subpage from the route parameter", async ({ Page, page, heading, section }) => {
    routeParams.page = page;
    render(() => <Page />);
    await waitFor(
      () => {
        expect(screen.getByRole("heading", { level: 1 }).textContent?.trim()).toBe(heading);
        expect(screen.getByRole("heading", { level: 2, name: new RegExp(section) })).not.toBeNull();
      },
      { timeout: 10_000 },
    );
  }, 30_000);
});
