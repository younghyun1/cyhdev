import { createLocalePage } from "./about-locales/lazy_locale_pages";

const About = createLocalePage({
  "en-US": () => import("./about-locales/about_me_en_us"),
  "ko-KR": () => import("./about-locales/about_me_ko_kr"),
  "fr-FR": () => import("./about-locales/about_me_fr_fr"),
  "es-ES": () => import("./about-locales/about_me_es_es"),
  "zh-Hans": () => import("./about-locales/about_me_zh_hans"),
  "zh-Hant": () => import("./about-locales/about_me_zh_hant"),
  "ja-JP": () => import("./about-locales/about_me_ja_jp"),
  "de-DE": () => import("./about-locales/about_me_de_de"),
});

export default About;
