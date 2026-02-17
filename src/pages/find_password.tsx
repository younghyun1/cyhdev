import { createSignal, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";

function FindPasswordPage() {
  const [email, setEmail] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [successMessage, setSuccessMessage] = createSignal<string | null>(null);
  const navigate = useNavigate();

  const handleFindPassword = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await authApi.resetPasswordRequest({
        user_email: email(),
      });

      setSuccessMessage(
        "If an account with that email exists, a password reset link has been sent.",
      );
      setEmail(""); // Clear the input on success
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
        <h2 class={`${pageStyles.titleSm} mb-6`}>Find Password</h2>
        <form
          onSubmit={handleFindPassword}
          class="w-full flex flex-col items-center"
        >
          <p class={`${pageStyles.muted} w-full mb-4 text-center`}>
            Enter your email to receive a password reset link.
          </p>
          <input
            type="email"
            placeholder="Email"
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
            class={`${pageStyles.input} mb-4`}
            autocomplete="email"
            required
          />

          <Show when={error()}>
            <div class={`${pageStyles.alertError} w-full mb-3 text-center`}>
              {error()}
            </div>
          </Show>

          <Show when={successMessage()}>
            <div class={`${pageStyles.alertSuccess} w-full mb-3 text-center`}>
              {successMessage()}
            </div>
          </Show>

          <button
            class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
            type="submit"
            disabled={loading() || !!successMessage()}
          >
            {loading() ? "Sending..." : "Send Reset Link"}
          </button>
          <button
            class={`${pageStyles.buttonSecondary} w-full py-3`}
            type="button"
            onClick={() => navigate("/login")}
          >
            Back to Login
          </button>
        </form>
      </div>
    </div>
  );
}

export default FindPasswordPage;
