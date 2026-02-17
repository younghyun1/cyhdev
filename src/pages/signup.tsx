import { createSignal, createEffect, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authApi, dropdownApi } from "../services/all_api";
import type {
  IsoCountry,
  IsoCountrySubdivision,
  IsoLanguage,
} from "../dtos/responses/dropdown";
import type { SignupResponse } from "../dtos/responses/auth";
import { pageStyles } from "../styles/pageStyles";

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
      setError("Please fill out all required fields.");
      return;
    }

    if (passwordsMismatch()) {
      setError("Passwords do not match.");
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
        setError("Signup failed.");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Signup failed.");
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
        <h2 class={`${pageStyles.titleSm} mb-6`}>Sign Up</h2>

        <Show
          when={!success()}
          fallback={
            <div class="text-center">
              <p class={`${pageStyles.alertSuccess} mb-4`}>
                Signup successful! Check your email to verify your account.
              </p>
              <button
                class={`${pageStyles.buttonPrimary} w-full py-2`}
                onClick={() => navigate("/login")}
              >
                Go to Login
              </button>
            </div>
          }
        >
          <form onSubmit={handleSubmit}>
            <input
              class={fieldClasses}
              type="text"
              placeholder="Username"
              value={userName()}
              onInput={(e) => setUserName(e.currentTarget.value)}
            />

            <input
              class={fieldClasses}
              type="email"
              placeholder="Email"
              autocomplete="username email"
              value={userEmail()}
              onInput={(e) => setUserEmail(e.currentTarget.value)}
            />

            <input
              class={fieldClasses}
              type="password"
              placeholder="Password"
              autocomplete="new-password"
              value={userPassword()}
              onInput={(e) => setUserPassword(e.currentTarget.value)}
              required
            />

            <input
              class={fieldClasses}
              type="password"
              placeholder="Re-enter Password"
              autocomplete="new-password"
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              required
              aria-invalid={passwordsMismatch() ? "true" : "false"}
            />

            <Show when={passwordsMismatch()}>
              <div class="mb-2 -mt-2 text-sm text-red-600 dark:text-red-400">
                Passwords do not match.
              </div>
            </Show>

            <select
              class={pageStyles.select + " mb-4"}
              value={userCountry()}
              onInput={(e) => setUserCountry(e.currentTarget.value)}
              required
            >
              <option value="">Select Country…</option>
              {countries().map((c) => (
                <option value={c.country_code}>
                  {c.country_flag ? c.country_flag + " " : ""}
                  {c.country_eng_name}
                </option>
              ))}
            </select>

            <select
              class={pageStyles.select + " mb-4"}
              value={userLanguage()}
              onInput={(e) => setUserLanguage(e.currentTarget.value)}
              required
            >
              <option value="">Select Language…</option>
              {sortedLanguages().map((l) => (
                <option value={l.language_code}>{l.language_eng_name}</option>
              ))}
            </select>

            <select
              class={`${pageStyles.select} mb-6`}
              value={userSubdivision()}
              onInput={(e) => setUserSubdivision(e.currentTarget.value)}
              disabled={!subdivisions().length}
            >
              <option value="">No Subdivision / N/A</option>
              {subdivisions().map((s) => (
                <option value={s.subdivision_id}>{s.subdivision_name}</option>
              ))}
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
              {loading() ? "Signing Up…" : "Sign Up"}
            </button>

            <button
              type="button"
              class={`${pageStyles.buttonSecondary} w-full py-3`}
              onClick={() => navigate("/login")}
            >
              Back to Login
            </button>
          </form>
        </Show>
      </div>
    </div>
  );
}

export default SignupPage;
