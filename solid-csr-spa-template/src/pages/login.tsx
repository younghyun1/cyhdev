import { createSignal, Show } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { authApi } from "../services/all_api";
import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import { consumePostLoginRedirect } from "../services/api";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

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
        setError(t("auth.login.failed"));
      }
    } catch {
      setAuthenticated(false);
      setUser(null);
      setSuperuser(false);
      setError(t("auth.login.failed"));
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
        <h2 class={`${pageStyles.titleSm} mb-6`}>{t("page.login.title")}</h2>
        <form onSubmit={handleLogin} class="w-full flex flex-col items-center">
          <input
            type="email"
            placeholder={t("common.email")}
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
            class={`${pageStyles.input} mb-4`}
            autocomplete="username"
            maxlength={254}
            required
          />
          <input
            type="password"
            placeholder={t("common.password")}
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
            class={`${pageStyles.input} mb-6`}
            autocomplete="current-password"
            maxlength={128}
            required
          />
          <div class="flex justify-end w-full mb-6">
            <button
              class={pageStyles.buttonGhost}
              tabindex={-1}
              type="button"
              onClick={() => navigate("/find-password")}
            >
              {t("auth.login.find_password")}
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
            {loading() ? t("auth.login.loading") : t("page.login.title")}
          </button>
          <button
            class={`${pageStyles.buttonSecondary} w-full py-3`}
            type="button"
            onClick={() => navigate("/register")}
          >
            {t("auth.login.register")}
          </button>
        </form>
      </div>
    </div>
  );
}

export default LoginPage;
