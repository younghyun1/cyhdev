import { createSignal, createEffect, onMount, Show, For } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authApi, dropdownApi } from "../services/all_api";
import type {
  IsoCountry,
  IsoCountrySubdivision,
  IsoLanguage,
} from "../dtos/responses/dropdown";
import type { SignupResponse } from "../dtos/responses/auth";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

function SignupPage() {
  // ––––– form state (all dropdowns as strings!)
  const [userName, setUserName] = createSignal("");
  const [userEmail, setUserEmail] = createSignal("");
  const [userPassword, setUserPassword] = createSignal("");
  const [confirmPassword, setConfirmPassword] = createSignal(""); // NEW
  const [userCountry, setUserCountry] = createSignal("");
  const [userLanguage, setUserLanguage] = createSignal("");
  const [userSubdivision, setUserSubdivision] = createSignal("");

  // ––––– status flags
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal<SignupResponse | null>(null);

  // ––––– dropdown data
  const [countries, setCountries] = createSignal<IsoCountry[]>([]);
  const [languages, setLanguages] = createSignal<IsoLanguage[]>([]);
  const [subdivisions, setSubdivisions] = createSignal<IsoCountrySubdivision[]>(
    [],
  );

  const navigate = useNavigate();

  // Derived helper: do the two passwords match (and are both non-empty)?
  const passwordsMismatch = () =>
    userPassword() !== "" &&
    confirmPassword() !== "" &&
    userPassword() !== confirmPassword();

  // Mirror backend validate_password_form (validations.rs): len>=8 + upper + lower + digit.
  const isPasswordValid = (pw: string) =>
    pw.length >= 8 && /[a-z]/.test(pw) && /[A-Z]/.test(pw) && /[0-9]/.test(pw);

  // ––––– fetch countries & languages once on mount
  onMount(() => {
    dropdownApi
      .countryList()
      .then((res) => {
        const arr = Array.isArray(res.data?.countries)
          ? res.data.countries
          : [];
        setCountries(arr);
      })
      .catch(() => {});

    dropdownApi
      .languageList()
      .then((res) => {
        const arr = Array.isArray(res.data) ? res.data : [];
        setLanguages(arr);
      })
      .catch(() => {});
  });

  // ––––– whenever userCountry changes, fetch subdivisions
  createEffect(() => {
    const cc = userCountry();
    if (cc) {
      dropdownApi
        .countrySubdivisions(Number(cc))
        .then((res) => {
          const arr = Array.isArray(res.data) ? res.data : [];
          setSubdivisions(arr);
        })
        .catch(() => {
          setSubdivisions([]);
        });
    } else {
      setSubdivisions([]);
    }
  });

  // ––––– on form submit
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);

    if (
      !userName() ||
      !userEmail() ||
      !userPassword() ||
      !userCountry() ||
      !userLanguage()
    ) {
      setError(t("auth.signup.required_fields"));
      return;
    }

    if (!isPasswordValid(userPassword())) {
      setError(t("auth.signup.password_rules"));
      return;
    }

    if (passwordsMismatch()) {
      setError(t("auth.signup.password_mismatch"));
      return;
    }

    setLoading(true);
    const body = {
      user_name: userName(),
      user_email: userEmail(),
      user_password: userPassword(),
      user_country: Number(userCountry()),
      user_language: Number(userLanguage()),
      user_subdivision: userSubdivision() ? Number(userSubdivision()) : null,
    };
    try {
      const res = await authApi.signup(body);
      if (res.success && res.data) {
        setSuccess(res.data);
      } else {
        setError(t("auth.signup.failed"));
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : t("auth.signup.failed"));
    } finally {
      setLoading(false);
    }
  };

  // ––––– sort languages: primary first by language_code, rest alphabetically by language_eng_name
  const sortedLanguages = () => {
    const all = [...languages()];
    const cc = Number(userCountry());
    const country = countries().find((c) => Number(c.country_code) === cc);
    const primaryCode = Number(country?.country_primary_language);

    // pull out the primary (if any)
    const primary = all.find((l) => Number(l.language_code) === primaryCode);

    // sort the rest by english name A→Z
    const rest = all
      .filter((l) => Number(l.language_code) !== primaryCode)
      .sort((a, b) => {
        const na = (a.language_eng_name ?? "").toLowerCase();
        const nb = (b.language_eng_name ?? "").toLowerCase();
        return na.localeCompare(nb);
      });

    return primary ? [primary, ...rest] : rest;
  };

  const fieldClasses = `${pageStyles.input} mb-4`;

  return (
    <div
      class={`${pageStyles.page} flex items-center justify-center px-6 py-10`}
    >
      <div class={`${pageStyles.card} w-full max-w-md p-8`}>
        <h2 class={`${pageStyles.titleSm} mb-6`}>{t("page.signup.title")}</h2>

        <Show
          when={!success()}
          fallback={
            <div class="text-center">
              <p class={`${pageStyles.alertSuccess} mb-4`}>
                {t("auth.signup.success")}
              </p>
              <button
                class={`${pageStyles.buttonPrimary} w-full py-2`}
                onClick={() => navigate("/login")}
              >
                {t("auth.signup.go_to_login")}
              </button>
            </div>
          }
        >
          <form onSubmit={handleSubmit}>
            <input
              class={fieldClasses}
              type="text"
              placeholder={t("common.username")}
              value={userName()}
              onInput={(e) => setUserName(e.currentTarget.value)}
            />

            <input
              class={fieldClasses}
              type="email"
              placeholder={t("common.email")}
              autocomplete="username email"
              value={userEmail()}
              onInput={(e) => setUserEmail(e.currentTarget.value)}
            />

            <input
              class={fieldClasses}
              type="password"
              placeholder={t("common.password")}
              autocomplete="new-password"
              value={userPassword()}
              onInput={(e) => setUserPassword(e.currentTarget.value)}
              required
            />

            <input
              class={fieldClasses}
              type="password"
              placeholder={t("auth.signup.reenter_password")}
              autocomplete="new-password"
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              required
              aria-invalid={passwordsMismatch() ? "true" : "false"}
            />

            <Show when={passwordsMismatch()}>
              <div class="mb-2 -mt-2 text-sm text-red-600 dark:text-red-400">
                {t("auth.signup.password_mismatch")}
              </div>
            </Show>

            <select
              class={pageStyles.select + " mb-4"}
              value={userCountry()}
              onInput={(e) => setUserCountry(e.currentTarget.value)}
              required
            >
              <option value="">{t("auth.signup.select_country")}</option>
              <For each={countries()}>{(c) => (
                <option value={c.country_code}>
                  {c.country_flag ? c.country_flag + " " : ""}
                  {c.country_eng_name}
                </option>
              )}</For>
            </select>

            <select
              class={pageStyles.select + " mb-4"}
              value={userLanguage()}
              onInput={(e) => setUserLanguage(e.currentTarget.value)}
              required
            >
              <option value="">{t("auth.signup.select_language")}</option>
              <For each={sortedLanguages()}>{(l) => (
                <option value={l.language_code}>{l.language_eng_name}</option>
              )}</For>
            </select>

            <select
              class={`${pageStyles.select} mb-6`}
              value={userSubdivision()}
              onInput={(e) => setUserSubdivision(e.currentTarget.value)}
              disabled={!subdivisions().length}
            >
              <option value="">{t("auth.signup.no_subdivision")}</option>
              <For each={subdivisions()}>{(s) => (
                <option value={s.subdivision_id}>{s.subdivision_name}</option>
              )}</For>
            </select>

            <Show when={error()}>
              <div class={`${pageStyles.alertError} mb-4 text-center`}>
                {error()}
              </div>
            </Show>

            <button
              type="submit"
              disabled={loading() || passwordsMismatch()}
              class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
            >
              {loading() ? t("auth.signup.loading") : t("page.signup.title")}
            </button>

            <button
              type="button"
              class={`${pageStyles.buttonSecondary} w-full py-3`}
              onClick={() => navigate("/login")}
            >
              {t("auth.signup.back_to_login")}
            </button>
          </form>
        </Show>
      </div>
    </div>
  );
}

export default SignupPage;
