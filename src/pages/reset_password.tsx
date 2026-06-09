import { createSignal, Show, onMount, onCleanup } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

function ResetPasswordPage() {
  const [searchParams] = useSearchParams();
  const [password, setPassword] = createSignal("");
  const [confirmPassword, setConfirmPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal(false);
  const navigate = useNavigate();
  let redirectTimer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => {
    if (redirectTimer !== undefined) clearTimeout(redirectTimer);
  });

  onMount(() => {
    if (!searchParams.token) {
      setError(t("auth.reset_password.missing_token_link"));
    }
  });

  const handleResetPassword = async (e: Event) => {
    e.preventDefault();
    const val = searchParams.token;
    const token = Array.isArray(val) ? val[0] : val;

    if (!token) {
      setError(t("auth.reset_password.missing_token"));
      return;
    }

    const pw = password();
    if (
      pw.length < 8 ||
      !/[a-z]/.test(pw) ||
      !/[A-Z]/.test(pw) ||
      !/[0-9]/.test(pw)
    ) {
      setError(t("auth.reset_password.too_short"));
      return;
    }

    if (password() !== confirmPassword()) {
      setError(t("auth.signup.password_mismatch"));
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await authApi.resetPassword({
        password_reset_token: token,
        new_password: password(),
      });
      setSuccess(true);
      // Optional: Automatically redirect after a few seconds
      redirectTimer = setTimeout(() => navigate("/login"), 3000);
    } catch (e: unknown) {
      let msg =
        e instanceof Error
          ? e.message
          : t("auth.find_password.unexpected_error");
      try {
        const json = JSON.parse(msg);
        if (json.message) msg = json.message;
      } catch {
        // ignore
      }
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      class={`${pageStyles.page} flex justify-center items-center px-6 py-10`}
    >
      <div
        class={`${pageStyles.card} w-full max-w-md p-8 flex flex-col items-center`}
      >
        <h2 class={`${pageStyles.titleSm} mb-6`}>
          {t("page.reset_password.title")}
        </h2>

        <Show when={success()}>
          <div class="w-full text-center">
            <div class={`${pageStyles.alertSuccess} w-full mb-6 text-center`}>
              {t("auth.reset_password.success")}
            </div>
            <p class={pageStyles.muted + " mb-6"}>
              {t("auth.reset_password.redirecting")}
            </p>
            <button
              class={`${pageStyles.buttonPrimary} w-full py-3`}
              onClick={() => navigate("/login")}
            >
              {t("auth.reset_password.go_now")}
            </button>
          </div>
        </Show>

        <Show when={!success()}>
          <form
            onSubmit={handleResetPassword}
            class="w-full flex flex-col items-center"
          >
            <p class={`${pageStyles.muted} w-full mb-4 text-center`}>
              {t("auth.reset_password.instructions")}
            </p>

            <input
              type="password"
              placeholder={t("auth.reset_password.new_password")}
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              class={`${pageStyles.input} mb-4`}
              required
              minlength={8}
            />

            <input
              type="password"
              placeholder={t("auth.reset_password.confirm_password")}
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              class={`${pageStyles.input} mb-4`}
              required
            />

            <Show when={error()}>
              <div class={`${pageStyles.alertError} w-full mb-3 text-center`}>
                {error()}
              </div>
            </Show>

            <button
              class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
              type="submit"
              disabled={loading() || !searchParams.token}
            >
              {loading()
                ? t("auth.reset_password.resetting")
                : t("page.reset_password.title")}
            </button>

            <button
              class={`${pageStyles.buttonSecondary} w-full py-3`}
              type="button"
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

export default ResetPasswordPage;
