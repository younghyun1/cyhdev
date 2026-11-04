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
  "ar": () => import("./about-locales/about_blog_ar"),
  "hi-IN": () => import("./about-locales/about_blog_hi_in"),
  "bn-BD": () => import("./about-locales/about_blog_bn_bd"),
  "te-IN": () => import("./about-locales/about_blog_te_in"),
  "pt-BR": () => import("./about-locales/about_blog_pt_br"),
  "id-ID": () => import("./about-locales/about_blog_id_id"),
  "ru-RU": () => import("./about-locales/about_blog_ru_ru"),
  "ur-PK": () => import("./about-locales/about_blog_ur_pk"),
  "vi-VN": () => import("./about-locales/about_blog_vi_vn"),
  "tl-PH": () => import("./about-locales/about_blog_tl_ph"),
  "fa-IR": () => import("./about-locales/about_blog_fa_ir"),
  "it-IT": () => import("./about-locales/about_blog_it_it"),
  "pl-PL": () => import("./about-locales/about_blog_pl_pl"),
  "sv-SE": () => import("./about-locales/about_blog_sv_se"),
  "sh": () => import("./about-locales/about_blog_sh"),
  "ro-RO": () => import("./about-locales/about_blog_ro_ro"),
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
