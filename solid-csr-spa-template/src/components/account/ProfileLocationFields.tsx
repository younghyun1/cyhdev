import { For, Show } from "solid-js";
import type {
  IsoCountry,
  IsoCountrySubdivision,
  IsoLanguage,
} from "../../generated";
import { t } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";

type ProfileLocationFieldsProps = {
  readonly countries: ReadonlyArray<IsoCountry>;
  readonly country: number;
  readonly languages: ReadonlyArray<IsoLanguage>;
  readonly language: number;
  readonly subdivisions: ReadonlyArray<IsoCountrySubdivision>;
  readonly subdivision: number | null;
  readonly disabled: boolean;
  readonly countryError?: string;
  readonly languageError?: string;
  readonly subdivisionError?: string;
  readonly onCountryChange: (value: number) => void;
  readonly onLanguageChange: (value: number) => void;
  readonly onSubdivisionChange: (value: number | null) => void;
};

function ErrorText(props: { readonly id: string; readonly value?: string }) {
  return (
    <Show when={props.value}>
      <p id={props.id} class="mt-1 text-sm text-danger" role="alert">
        {props.value}
      </p>
    </Show>
  );
}

export default function ProfileLocationFields(
  props: ProfileLocationFieldsProps,
) {
  return (
    <>
      <label class="block">
        <span class="mb-1 block text-sm font-medium">
          {t("common.country")}
        </span>
        <select
          class={pageStyles.select}
          value={String(props.country)}
          onChange={(event) =>
            props.onCountryChange(Number(event.currentTarget.value))
          }
          aria-describedby="profile-country-error"
          disabled={props.disabled}
        >
          <option value="0">{t("auth.signup.select_country")}</option>
          <For each={props.countries}>
            {(item) => (
              <option value={item.country_code}>
                {item.country_flag} {item.country_eng_name}
              </option>
            )}
          </For>
        </select>
        <ErrorText id="profile-country-error" value={props.countryError} />
      </label>
      <label class="block">
        <span class="mb-1 block text-sm font-medium">
          {t("common.subdivision")}
        </span>
        <select
          class={pageStyles.select}
          value={props.subdivision === null ? "" : String(props.subdivision)}
          onChange={(event) => {
            const value = event.currentTarget.value;
            props.onSubdivisionChange(value === "" ? null : Number(value));
          }}
          aria-describedby="profile-subdivision-error"
          disabled={props.disabled || props.country <= 0}
        >
          <option value="">{t("auth.signup.no_subdivision")}</option>
          <For each={props.subdivisions}>
            {(item) => (
              <option value={item.subdivision_id}>
                {item.subdivision_name}
              </option>
            )}
          </For>
        </select>
        <ErrorText
          id="profile-subdivision-error"
          value={props.subdivisionError}
        />
      </label>
      <label class="block">
        <span class="mb-1 block text-sm font-medium">
          {t("common.language")}
        </span>
        <select
          class={pageStyles.select}
          value={String(props.language)}
          onChange={(event) =>
            props.onLanguageChange(Number(event.currentTarget.value))
          }
          aria-describedby="profile-language-error"
          disabled={props.disabled}
        >
          <option value="0">{t("auth.signup.select_language")}</option>
          <For each={props.languages}>
            {(item) => (
              <option value={item.language_code}>
                {item.language_eng_name}
              </option>
            )}
          </For>
        </select>
        <ErrorText id="profile-language-error" value={props.languageError} />
      </label>
    </>
  );
}
