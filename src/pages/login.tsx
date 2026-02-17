import { createSignal, Show } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import { consumePostLoginRedirect } from "../services/api";
import { pageStyles } from "../styles/pageStyles";

function LoginPage() {
  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const res = await authApi.login({
        user_email: email(),
        user_password: password(),
      });
      if (res.success) {
        // Set auth state
        const meResp = await authApi.me();
        if (meResp?.success && meResp.data) {
          const hasUser = !!meResp.data.user_info?.user_id;
          setAuthenticated(hasUser);
          setUser(hasUser ? meResp.data : null);
          if (hasUser) {
            try {
              const superuserResp = await authApi.isSuperuser();
              setSuperuser(!!superuserResp.data?.is_superuser);
            } catch {
              setSuperuser(false);
            }
          }
        }

        // Redirect: prefer sessionStorage (set by 401 interceptor), fall back to ?next param
        const savedRedirect = consumePostLoginRedirect();
        const nextParam = Array.isArray(searchParams.next)
          ? searchParams.next[0]
          : searchParams.next;
        const target = savedRedirect || nextParam || "/";
        navigate(target, { replace: true });
        return;
      } else {
        setAuthenticated(false);
        setUser(null);
        setSuperuser(false);
        setError("Login failed");
      }
    } catch (err: unknown) {
      setAuthenticated(false);
      setUser(null);
      setSuperuser(false);
      setError(err instanceof Error ? err.message : "Login failed");
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
