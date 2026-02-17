import { createSignal, Show, onMount } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

function ResetPasswordPage() {
  const [searchParams] = useSearchParams();
  const [password, setPassword] = createSignal("");
  const [confirmPassword, setConfirmPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal(false);
  const navigate = useNavigate();

  onMount(() => {
    if (!searchParams.token) {
      setError("Missing password reset token. Please check your email link.");
    }
  });

  const handleResetPassword = async (e: Event) => {
    e.preventDefault();
    const val = searchParams.token;
    const token = Array.isArray(val) ? val[0] : val;

    if (!token) {
      setError("Missing password reset token.");
      return;
    }

    if (password().length < 8) {
      setError("Password must be at least 8 characters long.");
      return;
    }

    if (password() !== confirmPassword()) {
      setError("Passwords do not match.");
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
      setTimeout(() => navigate("/login"), 3000);
    } catch (e: unknown) {
      let msg =
        e instanceof Error
          ? e.message
          : "An unexpected error occurred. Please try again.";
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
        <h2 class={`${pageStyles.titleSm} mb-6`}>Reset Password</h2>

        <Show when={success()}>
          <div class="w-full text-center">
            <div class={`${pageStyles.alertSuccess} w-full mb-6 text-center`}>
              Your password has been successfully reset.
            </div>
            <p class={pageStyles.muted + " mb-6"}>Redirecting to login...</p>
            <button
              class={`${pageStyles.buttonPrimary} w-full py-3`}
              onClick={() => navigate("/login")}
            >
              Go to Login Now
            </button>
          </div>
        </Show>

        <Show when={!success()}>
          <form
            onSubmit={handleResetPassword}
            class="w-full flex flex-col items-center"
          >
            <p class={`${pageStyles.muted} w-full mb-4 text-center`}>
              Enter your new password below.
            </p>

            <input
              type="password"
              placeholder="New Password"
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              class={`${pageStyles.input} mb-4`}
              required
              minlength={8}
            />

            <input
              type="password"
              placeholder="Confirm Password"
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
              {loading() ? "Resetting..." : "Reset Password"}
            </button>

            <button
              class={`${pageStyles.buttonSecondary} w-full py-3`}
              type="button"
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

export default ResetPasswordPage;
