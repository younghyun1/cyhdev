import { Match, Switch } from "solid-js";
import { locale } from "../state/i18n";
import AboutMeEnUs from "./about-locales/about_me_en_us";
import AboutMeKoKr from "./about-locales/about_me_ko_kr";
import AboutMeFrFr from "./about-locales/about_me_fr_fr";
import AboutMeEsEs from "./about-locales/about_me_es_es";
import AboutMeZhHans from "./about-locales/about_me_zh_hans";
import AboutMeZhHant from "./about-locales/about_me_zh_hant";
import AboutMeJaJp from "./about-locales/about_me_ja_jp";
import AboutMeDeDe from "./about-locales/about_me_de_de";

export default function About() {
  return (
    <Switch fallback={<AboutMeEnUs />}>
      <Match when={locale() === "ko-KR"}><AboutMeKoKr /></Match>
      <Match when={locale() === "fr-FR"}><AboutMeFrFr /></Match>
      <Match when={locale() === "es-ES"}><AboutMeEsEs /></Match>
      <Match when={locale() === "zh-Hans"}><AboutMeZhHans /></Match>
      <Match when={locale() === "zh-Hant"}><AboutMeZhHant /></Match>
      <Match when={locale() === "ja-JP"}><AboutMeJaJp /></Match>
      <Match when={locale() === "de-DE"}><AboutMeDeDe /></Match>
    </Switch>
  );
}
