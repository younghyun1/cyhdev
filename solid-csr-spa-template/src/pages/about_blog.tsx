import { createLocalePage } from "./about-locales/lazy_locale_pages";

const AboutBlog = createLocalePage({
  "en-US": () => import("./about-locales/about_blog_en_us"),
  "ko-KR": () => import("./about-locales/about_blog_ko_kr"),
  "fr-FR": () => import("./about-locales/about_blog_fr_fr"),
  "es-ES": () => import("./about-locales/about_blog_es_es"),
  "zh-Hans": () => import("./about-locales/about_blog_zh_hans"),
  "zh-Hant": () => import("./about-locales/about_blog_zh_hant"),
  "ja-JP": () => import("./about-locales/about_blog_ja_jp"),
  "de-DE": () => import("./about-locales/about_blog_de_de"),
});

export default AboutBlog;
