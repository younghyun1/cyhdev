import { Match, Switch } from "solid-js";
import { locale } from "../state/i18n";
import AboutBlogEnUs from "./about-locales/about_blog_en_us";
import AboutBlogKoKr from "./about-locales/about_blog_ko_kr";
import AboutBlogFrFr from "./about-locales/about_blog_fr_fr";
import AboutBlogEsEs from "./about-locales/about_blog_es_es";
import AboutBlogZhHans from "./about-locales/about_blog_zh_hans";
import AboutBlogZhHant from "./about-locales/about_blog_zh_hant";
import AboutBlogJaJp from "./about-locales/about_blog_ja_jp";
import AboutBlogDeDe from "./about-locales/about_blog_de_de";

export default function AboutBlog() {
  return (
    <Switch fallback={<AboutBlogEnUs />}>
      <Match when={locale() === "ko-KR"}><AboutBlogKoKr /></Match>
      <Match when={locale() === "fr-FR"}><AboutBlogFrFr /></Match>
      <Match when={locale() === "es-ES"}><AboutBlogEsEs /></Match>
      <Match when={locale() === "zh-Hans"}><AboutBlogZhHans /></Match>
      <Match when={locale() === "zh-Hant"}><AboutBlogZhHant /></Match>
      <Match when={locale() === "ja-JP"}><AboutBlogJaJp /></Match>
      <Match when={locale() === "de-DE"}><AboutBlogDeDe /></Match>
    </Switch>
  );
}
