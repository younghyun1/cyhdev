import { Show, createMemo } from "solid-js";
import { useParams } from "@solidjs/router";
import ContentPage from "../components/content/ContentPage";
import { RepositorySourceMap } from "../components/RepositorySourceMap";
import { locale } from "../state/i18n";
import {
  ABOUT_BLOG_PAGE_IDS,
  type AboutBlogContent,
  type AboutBlogPageId,
} from "./about-locales/content_types";
import { createLocaleContentLoader } from "./about-locales/locale_content";

const loadAboutBlog = createLocaleContentLoader<AboutBlogContent>({
  "en-US": () => import("./about-locales/about_blog_en_us"),
  "ko-KR": () => import("./about-locales/about_blog_ko_kr"),
  "fr-FR": () => import("./about-locales/about_blog_fr_fr"),
  "es-ES": () => import("./about-locales/about_blog_es_es"),
  "zh-Hans": () => import("./about-locales/about_blog_zh_hans"),
  "zh-Hant": () => import("./about-locales/about_blog_zh_hant"),
  "ja-JP": () => import("./about-locales/about_blog_ja_jp"),
  "de-DE": () => import("./about-locales/about_blog_de_de"),
});

/** Subpage ids accepted by the `/about-blog/:page` route filter. */
export const ABOUT_BLOG_SUBPAGES = ABOUT_BLOG_PAGE_IDS.slice(1);

function toPageId(value: string | undefined): AboutBlogPageId {
  return (ABOUT_BLOG_PAGE_IDS as ReadonlyArray<string>).includes(value ?? "")
    ? (value as AboutBlogPageId)
    : "overview";
}

/** About Blog hub and subpages; see `about.tsx` for the loading model. */
export default function AboutBlog() {
  const params = useParams<{ page?: string }>();
  const pageId = createMemo(() => toPageId(params.page));
  const loaded = createMemo(() => loadAboutBlog(locale()));

  return (
    <Show when={`${loaded().locale}:${pageId()}`} keyed>
      <ContentPage
        locale={loaded().locale}
        content={loaded().content}
        pageId={pageId()}
        pageIds={ABOUT_BLOG_PAGE_IDS}
        basePath="/about-blog"
      >
        <Show when={pageId() === "overview"}>
          <RepositorySourceMap />
        </Show>
      </ContentPage>
    </Show>
  );
}
