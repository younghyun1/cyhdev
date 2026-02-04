import { createSignal, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import { pageStyles } from "../styles/pageStyles";

function LoginPage() {
  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const navigate = useNavigate();

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams(window.location.search);
      const next = params.get("next");
      const saved = sessionStorage.getItem("post_login_redirect");
      const target = next || saved || "/";
      sessionStorage.setItem("post_login_redirect", target);

      const res = await authApi.login({
        user_email: email(),
        user_password: password(),
      });
      if (res.success) {
        // Redirect is handled globally in apiFetch using sessionStorage.post_login_redirect
        return;
      } else {
        setAuthenticated(false);
        setUser(null);
        setSuperuser(false);
        setError(res?.data?.message ?? "Login failed");
      }
    } catch (e: any) {
      setAuthenticated(false);
      setUser(null);
      setSuperuser(false);
      setError(e?.message ?? "Login failed");
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
        <h2 class={`${pageStyles.titleSm} mb-6`}>Login</h2>
        <form onSubmit={handleLogin} class="w-full flex flex-col items-center">
          <input
            type="email"
            placeholder="Email"
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
            class={`${pageStyles.input} mb-4`}
            autocomplete="username"
            required
          />
          <input
            type="password"
            placeholder="Password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
            class={`${pageStyles.input} mb-6`}
            autocomplete="current-password"
            required
          />
          <div class="flex justify-end w-full mb-6">
            <button
              class={pageStyles.buttonGhost}
              tabIndex={-1}
              type="button"
              onClick={() => navigate("/find-password")}
            >
              Find Password
            </button>
          </div>
          <Show when={error()}>
            <div class={`${pageStyles.alertError} w-full mb-3 text-center`}>
              {error()}
            </div>
          </Show>
          <button
            class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
            type="submit"
            disabled={loading()}
          >
            {loading() ? "Logging in..." : "Login"}
          </button>
          <button
            class={`${pageStyles.buttonSecondary} w-full py-3`}
            type="button"
            onClick={() => navigate("/register")}
          >
            Register
          </button>
        </form>
      </div>
    </div>
  );
}

export default LoginPage;
