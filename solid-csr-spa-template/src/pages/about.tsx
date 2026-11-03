import { Show, createMemo } from "solid-js";
import { useParams } from "@solidjs/router";
import ContentPage from "../components/content/ContentPage";
import { locale } from "../state/i18n";
import {
  ABOUT_ME_PAGE_IDS,
  type AboutMeContent,
  type AboutMePageId,
} from "./about-locales/content_types";
import { createLocaleContentLoader } from "./about-locales/locale_content";

const loadAboutMe = createLocaleContentLoader<AboutMeContent>({
  "en-US": () => import("./about-locales/about_me_en_us"),
  "ko-KR": () => import("./about-locales/about_me_ko_kr"),
  "fr-FR": () => import("./about-locales/about_me_fr_fr"),
  "es-ES": () => import("./about-locales/about_me_es_es"),
  "zh-Hans": () => import("./about-locales/about_me_zh_hans"),
  "zh-Hant": () => import("./about-locales/about_me_zh_hant"),
  "ja-JP": () => import("./about-locales/about_me_ja_jp"),
  "de-DE": () => import("./about-locales/about_me_de_de"),
});

/** Subpage ids accepted by the `/about/:page` route filter. */
export const ABOUT_ME_SUBPAGES = ABOUT_ME_PAGE_IDS.slice(1);

function toPageId(value: string | undefined): AboutMePageId {
  return (ABOUT_ME_PAGE_IDS as ReadonlyArray<string>).includes(value ?? "")
    ? (value as AboutMePageId)
    : "overview";
}

function Portrait() {
  return (
    <figure class="cp-portrait">
      <a
        href="https://cyhdev-img.s3.us-west-1.amazonaws.com/images/05da63f0-0a8e-4d96-807f-280ded45a6d5.avif"
        target="_blank"
        rel="noopener noreferrer"
      >
        <img
          src="https://cyhdev-img.s3.us-west-1.amazonaws.com/thumbnails/05da63f0-0a8e-4d96-807f-280ded45a6d5.avif"
          alt="Portrait of Young Hyun Chi"
          width="160"
          height="160"
          decoding="async"
        />
      </a>
    </figure>
  );
}

/**
 * About Me hub and subpages. Copy comes from one typed module per locale; the
 * page id comes from the route parameter, and the frame remounts per locale
 * and page so its scroll-spy and entry animation start fresh.
 */
export default function About() {
  const params = useParams<{ page?: string }>();
  const pageId = createMemo(() => toPageId(params.page));
  const loaded = createMemo(() => loadAboutMe(locale()));

  return (
    <Show when={`${loaded().locale}:${pageId()}`} keyed>
      <ContentPage
        locale={loaded().locale}
        content={loaded().content}
        pageId={pageId()}
        pageIds={ABOUT_ME_PAGE_IDS}
        basePath="/about"
        pageClass="about-page"
        aside={pageId() === "overview" ? <Portrait /> : undefined}
      />
    </Show>
  );
}
