import { For, type Component } from "solid-js";
import { isUiLocale, UI_LOCALES } from "../i18n/locales";
import { locale, setLocale, t } from "../state/i18n";

const LanguageSelect: Component = () => {
  const handleChange = (event: Event) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (isUiLocale(value)) {
      void setLocale(value);
    }
  };

  return (
    <select
      class="h-8 rounded-sm border border-line bg-surface/80 px-2 text-xs text-ink hover:bg-surface-2 transition-colors"
      value={locale()}
      aria-label={t("top_bar.language.label")}
      onChange={handleChange}
    >
      <For each={UI_LOCALES}>{(language) => <option value={language.tag} lang={language.tag}>{language.label}</option>}</For>
    </select>
  );
};

export default LanguageSelect;
