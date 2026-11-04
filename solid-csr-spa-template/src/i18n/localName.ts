import type { UiLocale } from "./keys";

/**
 * The author's name for scripts the Latin, Korean, and Chinese renditions do
 * not cover. These are transliterations of the name's Korean sound, not
 * translations, so they live here rather than in the interface catalogs.
 */
export const LOCAL_NAMES: Partial<Readonly<Record<UiLocale, string>>> = {
  ar: "يونغهيون تشي",
  "hi-IN": "यंगह्यून ची",
  "bn-BD": "ইয়ংহিউন চি",
  "te-IN": "యంగ్‌హ్యూన్ చి",
  "ru-RU": "Ёнхён Чи",
  "ur-PK": "یونگ ہیون چی",
  "fa-IR": "یونگ‌هیون چی",
};
