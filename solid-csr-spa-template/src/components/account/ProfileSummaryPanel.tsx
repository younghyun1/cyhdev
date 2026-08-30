import { Show, createEffect, createSignal, onSettled, untrack } from "solid-js";
import { ApiContractError } from "../../generated";
import type { IsoCountry, IsoCountrySubdivision, IsoLanguage } from "../../generated";
import { authApi, dropdownApi } from "../../services/all_api";
import { setUser, user } from "../../state/auth";
import { t } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";
import ProfileFieldError from "./ProfileFieldError";
import ProfileLocationFields from "./ProfileLocationFields";

type ProfileField = "currentPassword" | "userName" | "country" | "language" | "subdivision";
type FieldErrors = Partial<Record<ProfileField, string>>;

export default function ProfileSummaryPanel() {
  const initial = untrack(() => user()?.user_info);
  const [userName, setUserName] = createSignal(initial?.user_name ?? "");
  const [country, setCountry] = createSignal(initial?.user_country ?? 0);
  const [language, setLanguage] = createSignal(initial?.user_language ?? 0);
  const [subdivision, setSubdivision] = createSignal<number | null>(
    initial?.user_subdivision ?? null,
  );
  const [currentPassword, setCurrentPassword] = createSignal("");
  const [countries, setCountries] = createSignal<ReadonlyArray<IsoCountry>>([]);
  const [languages, setLanguages] = createSignal<ReadonlyArray<IsoLanguage>>([]);
  const [subdivisions, setSubdivisions] = createSignal<
    ReadonlyArray<IsoCountrySubdivision>
  >([]);
  const [optionsLoaded, setOptionsLoaded] = createSignal(false);
  const [pending, setPending] = createSignal(false);
  const [fieldErrors, setFieldErrors] = createSignal<FieldErrors>({});
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  const clearFieldError = (field: ProfileField) => {
    setFieldErrors((current) => {
      const next = { ...current };
      delete next[field];
      return next;
    });
    setSaved(false);
  };

  onSettled(() => {
    Promise.all([dropdownApi.countryList(), dropdownApi.languageList()])
      .then(([countryResponse, languageResponse]) => {
        setCountries(
          Array.isArray(countryResponse.data.countries)
            ? countryResponse.data.countries
            : [],
        );
        setLanguages(
          Array.isArray(languageResponse.data) ? languageResponse.data : [],
        );
        setOptionsLoaded(true);
      })
      .catch(() => setError(t("profile.update.options_failed")));
  });

  createEffect(
    () => country(),
    (countryId) => {
      if (countryId <= 0) {
        setSubdivisions([]);
        setSubdivision(null);
        return;
      }
      dropdownApi
        .countrySubdivisions(countryId)
        .then((response) => {
          if (country() !== countryId) return;
          const values = Array.isArray(response.data) ? response.data : [];
          setSubdivisions(values);
          const selected = subdivision();
          if (
            selected !== null &&
            !values.some((item) => item.subdivision_id === selected)
          ) {
            setSubdivision(null);
          }
        })
        .catch(() => {
          if (country() === countryId) setSubdivisions([]);
        });
    },
  );

  const validate = (): FieldErrors => {
    const errors: FieldErrors = {};
    const name = userName().trim();
    if (
      name.length === 0 ||
      [...name].length > 20 ||
      !/^[\p{L}\p{N}]+$/u.test(name)
    ) {
      errors.userName = t("profile.update.name_invalid");
    }
    if (!countries().some((item) => item.country_code === country())) {
      errors.country = t("profile.update.country_required");
    }
    if (!languages().some((item) => item.language_code === language())) {
      errors.language = t("profile.update.language_required");
    }
    const selectedSubdivision = subdivision();
    if (
      selectedSubdivision !== null &&
      !subdivisions().some(
        (item) => item.subdivision_id === selectedSubdivision,
      )
    ) {
      errors.subdivision = t("profile.update.subdivision_invalid");
    }
    if (currentPassword().length === 0) {
      errors.currentPassword = t("profile.update.password_required");
    }
    return errors;
  };

  const handleSubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (pending()) return;
    if (!optionsLoaded()) {
      setError(t("profile.update.options_failed"));
      return;
    }
    const validation = validate();
    setFieldErrors(validation);
    if (Object.keys(validation).length > 0) return;

    setPending(true);
    setError(null);
    setSaved(false);
    try {
      const response = await authApi.updateProfile({
        current_password: currentPassword(),
        user_name: userName().trim(),
        user_country: country(),
        user_language: language(),
        user_subdivision: subdivision(),
      });
      if (!response.success) {
        setError(t("profile.update.failed"));
        return;
      }
      const updated = response.data;
      setUserName(updated.user_name);
      setCountry(updated.user_country);
      setLanguage(updated.user_language);
      setSubdivision(updated.user_subdivision ?? null);
      const current = user();
      if (current?.user_info) {
        setUser({
          ...current,
          user_info: {
            ...current.user_info,
            user_name: updated.user_name,
            user_country: updated.user_country,
            user_language: updated.user_language,
            user_subdivision: updated.user_subdivision ?? null,
          },
        });
      }
      setSaved(true);
    } catch (caught: unknown) {
      if (caught instanceof ApiContractError && caught.status === 409) {
        setFieldErrors({ userName: t("profile.update.name_conflict") });
      } else if (caught instanceof ApiContractError && caught.status === 422) {
        setFieldErrors({
          currentPassword: t("profile.update.password_wrong"),
        });
      } else {
        setError(
          caught instanceof Error ? caught.message : t("profile.update.failed"),
        );
      }
    } finally {
      setCurrentPassword("");
      setPending(false);
    }
  };

  return (
    <section class={pageStyles.cardPadded}>
      <h2 class="text-lg font-semibold">{t("profile.update.title")}</h2>
      <hr class={`my-3 ${pageStyles.divider}`} />
      <p class={`${pageStyles.muted} mb-4`}>
        {t("profile.update.description")}
      </p>
      <form class="space-y-4" onSubmit={handleSubmit}>
        <label class="block">
          <span class="mb-1 block text-sm font-medium">
            {t("common.email")}
          </span>
          <input
            class={`${pageStyles.input} bg-surface-2`}
            value={initial?.user_email ?? ""}
            readOnly
            aria-readonly="true"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-sm font-medium">
            {t("profile.display_name")}
          </span>
          <input
            class={pageStyles.input}
            value={userName()}
            maxlength={20}
            onInput={(event) => {
              setUserName(event.currentTarget.value);
              clearFieldError("userName");
            }}
            aria-describedby="profile-name-error"
            disabled={pending()}
            required
          />
          <ProfileFieldError
            id="profile-name-error"
            message={fieldErrors().userName}
          />
        </label>
        <ProfileLocationFields
          countries={countries()}
          country={country()}
          languages={languages()}
          language={language()}
          subdivisions={subdivisions()}
          subdivision={subdivision()}
          disabled={pending() || !optionsLoaded()}
          countryError={fieldErrors().country}
          languageError={fieldErrors().language}
          subdivisionError={fieldErrors().subdivision}
          onCountryChange={(value) => {
            setCountry(value);
            setSubdivision(null);
            clearFieldError("country");
            clearFieldError("subdivision");
          }}
          onLanguageChange={(value) => {
            setLanguage(value);
            clearFieldError("language");
          }}
          onSubdivisionChange={(value) => {
            setSubdivision(value);
            clearFieldError("subdivision");
          }}
        />
        <label class="block">
          <span class="mb-1 block text-sm font-medium">
            {t("profile.update.current_password")}
          </span>
          <input
            class={pageStyles.input}
            type="password"
            autocomplete="current-password"
            value={currentPassword()}
            onInput={(event) => {
              setCurrentPassword(event.currentTarget.value);
              clearFieldError("currentPassword");
            }}
            aria-describedby="profile-password-error"
            disabled={pending()}
            required
          />
          <ProfileFieldError
            id="profile-password-error"
            message={fieldErrors().currentPassword}
          />
        </label>
        <Show when={error()}>
          <div class={pageStyles.alertError} role="alert">
            {error()}
          </div>
        </Show>
        <Show when={saved()}>
          <div class={pageStyles.alertSuccess} aria-live="polite">
            {t("profile.update.success")}
          </div>
        </Show>
        <button
          class={pageStyles.buttonPrimary}
          type="submit"
          disabled={pending() || !optionsLoaded()}
        >
          {pending() ? t("common.saving") : t("common.save")}
        </button>
      </form>
    </section>
  );
}
