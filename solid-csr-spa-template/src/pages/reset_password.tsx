import { createSignal, Show, onSettled, onCleanup } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

const CANONICAL_UUID =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function ResetPasswordPage() {
  const [resetToken, setResetToken] = createSignal<string | null>(null);
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

  onSettled(() => {
    if (typeof window === "undefined") {
      setError(t("auth.reset_password.missing_token_link"));
      return;
    }
    const rawFragment = window.location.hash.startsWith("#")
      ? window.location.hash.slice(1)
      : window.location.hash;
    const entries = [...new URLSearchParams(rawFragment).entries()];
    const onlyEntry = entries.length === 1 ? entries[0] : undefined;
    const token = onlyEntry?.[0] === "token" ? onlyEntry[1] : undefined;
    window.history.replaceState(
      window.history.state,
      "",
      window.location.pathname,
    );
    if (token !== undefined && CANONICAL_UUID.test(token)) {
      setResetToken(token);
    } else {
      setError(t("auth.reset_password.missing_token_link"));
    }
  });

  const handleResetPassword = async (e: Event) => {
    e.preventDefault();
    const token = resetToken();

    if (!token) {
      setError(t("auth.reset_password.missing_token"));
      return;
    }

    const pw = password();
    const passwordBytes = new TextEncoder().encode(pw).byteLength;
    if (
      pw.length < 8 ||
      passwordBytes > 128 ||
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
      setResetToken(null);
      setSuccess(true);
      // Optional: Automatically redirect after a few seconds
      redirectTimer = setTimeout(() => navigate("/login"), 3000);
    } catch {
      setError(t("auth.find_password.unexpected_error"));
    } finally {
      setPassword("");
      setConfirmPassword("");
      setLoading(false);
    }
  };

  return (
    <div
      class={`${pageStyles.page} auth-page flex justify-center items-center px-6 py-10`}
    >
      <div
        class={`${pageStyles.card} auth-card w-full max-w-md p-8 flex flex-col items-center`}
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
              maxlength={128}
              autocomplete="new-password"
            />

            <input
              type="password"
              placeholder={t("auth.reset_password.confirm_password")}
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              class={`${pageStyles.input} mb-4`}
              required
              maxlength={128}
              autocomplete="new-password"
            />

            <Show when={error()}>
              <div class={`${pageStyles.alertError} w-full mb-3 text-center`}>
                {error()}
              </div>
            </Show>

            <button
              class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
              type="submit"
              disabled={loading() || !resetToken()}
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
