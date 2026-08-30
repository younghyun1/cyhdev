import type { Component } from "solid-js";
import type { UiLocale } from "../i18n/keys";
import { locale, setLocale, t } from "../state/i18n";

const LanguageSelect: Component = () => {
  const handleChange = (event: Event) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (value === "en-US" || value === "ko-KR") {
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
      <option value={"en-US" satisfies UiLocale}>
        {t("top_bar.language.english")}
      </option>
      <option value={"ko-KR" satisfies UiLocale}>
        {t("top_bar.language.korean")}
      </option>
    </select>
  );
};

export default LanguageSelect;
