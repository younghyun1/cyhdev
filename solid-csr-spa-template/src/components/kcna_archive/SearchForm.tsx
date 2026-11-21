import { createEffect, createSignal, type Component } from "solid-js";

import { t } from "../../state/i18n";
import { MAX_TEXT_LENGTH } from "../../utils/kcna_archive/query";

/** The text search field. The URL owns the active text; the field follows it. */
const SearchForm: Component<{
  readonly text: string;
  readonly onSearch: (text: string) => void;
}> = (props) => {
  const [draft, setDraft] = createSignal("");
  createEffect(
    () => props.text,
    (text) => {
      setDraft(text);
    },
  );

  return (
    <form
      class="kcna-search"
      role="search"
      onSubmit={(event) => {
        event.preventDefault();
        props.onSearch(draft().trim().slice(0, MAX_TEXT_LENGTH));
      }}
    >
      <label class="sr-only" for="kcna-search-input">
        {t("kcna.search.placeholder")}
      </label>
      <input
        id="kcna-search-input"
        class="kcna-input"
        type="search"
        dir="auto"
        data-kcna-primary-input
        maxlength={MAX_TEXT_LENGTH}
        placeholder={t("kcna.search.placeholder")}
        autocomplete="off"
        value={draft()}
        onInput={(event) => setDraft(event.currentTarget.value)}
      />
      <button type="submit" class="kcna-button kcna-button--primary">
        {t("kcna.search.submit")}
      </button>
    </form>
  );
};

export default SearchForm;
