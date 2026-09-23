import { createSignal, onSettled, Show } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { authApi, oidcApi } from "../services/all_api";
import type { OidcStatusResponse } from "../generated";
import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import {
  consumePostLoginRedirect,
  rememberPostLoginRedirect,
  safeRedirectTarget,
} from "../services/api";
import { consumeOidcFragment } from "../services/oidc_fragment";
import {
  LOGIN_EMAIL_NOT_VERIFIED,
  apiErrorCode,
} from "../services/api_error_code";
import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";

function LoginPage() {
  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [oidcStatus, setOidcStatus] = createSignal<OidcStatusResponse | null>(
    null,
  );
  const [error, setError] = createSignal<string | null>(null);
  // A correct password for an unverified account: no session exists until the owner
  // follows the emailed link, and signing up again is the way to get a fresh one.
  const [verificationRequired, setVerificationRequired] = createSignal(false);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const requestedTarget = () => {
    const next = Array.isArray(searchParams.next)
      ? searchParams.next[0]
      : searchParams.next;
    return safeRedirectTarget(next);
  };

  const hydrateSession = async (): Promise<boolean> => {
    const meResp = await authApi.me();
    const hasUser = !!meResp.data?.user_info?.user_id;
    setAuthenticated(hasUser);
    setUser(hasUser ? meResp.data : null);
    if (!hasUser) {
      setSuperuser(false);
      return false;
    }
    try {
      const superuserResp = await authApi.isSuperuser();
      setSuperuser(!!superuserResp.data?.is_superuser);
    } catch {
      setSuperuser(false);
    }
    return true;
  };

  const finishLogin = async () => {
    if (!(await hydrateSession())) {
      throw new Error("authenticated session was not established");
    }
    const target = consumePostLoginRedirect() || requestedTarget() || "/";
    navigate(target, { replace: true });
  };

  onSettled(() => {
    const callback = consumeOidcFragment();
    oidcApi
      .status()
      .then((response) => setOidcStatus(response.data ?? null))
      .catch(() => setOidcStatus(null));
    if (callback.kind === "failed") {
      setError(t("auth.oidc.failed"));
    } else if (callback.kind === "login-success") {
      setLoading(true);
      finishLogin()
        .catch(() => setError(t("auth.oidc.failed")))
        .finally(() => setLoading(false));
    }
  });

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setVerificationRequired(false);
    try {
      const res = await authApi.login({
        user_email: email(),
        user_password: password(),
      });
      if (res.success) {
        await finishLogin();
        return;
      } else {
        setAuthenticated(false);
        setUser(null);
        setSuperuser(false);
        setError(t("auth.login.failed"));
      }
    } catch (caught: unknown) {
      setAuthenticated(false);
      setUser(null);
      setSuperuser(false);
      if (apiErrorCode(caught) === LOGIN_EMAIL_NOT_VERIFIED) {
        setVerificationRequired(true);
      } else {
        setError(t("auth.login.failed"));
      }
    } finally {
      setLoading(false);
    }
  };

  const handleOidcLogin = async () => {
    setLoading(true);
    setError(null);
    const target = requestedTarget();
    if (target) rememberPostLoginRedirect(target);
    try {
      const response = await oidcApi.startLogin();
      const authorizationUrl = response.data?.authorization_url;
      if (!response.success || !authorizationUrl) {
        throw new Error("authorization URL missing");
      }
      window.location.assign(authorizationUrl);
    } catch {
      setError(t("auth.oidc.failed"));
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
          <Show when={verificationRequired()}>
            <div class={`${pageStyles.cardPadded} w-full mb-3`} role="status">
              <p class="mb-2">{t("auth.login.verification_required")}</p>
              <p class={`${pageStyles.muted} mb-3`}>
                {t("auth.login.verification_resend_hint")}
              </p>
              <button
                class={`${pageStyles.buttonSecondary} w-full`}
                type="button"
                onClick={() => navigate("/register")}
              >
                {t("auth.login.verification_resend")}
              </button>
            </div>
          </Show>
          <button
            class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
            type="submit"
            disabled={loading()}
          >
            {loading() ? t("auth.login.loading") : t("page.login.title")}
          </button>
          <Show when={oidcStatus()?.enabled}>
            <button
              class={`${pageStyles.buttonSecondary} w-full mb-3 py-3`}
              type="button"
              disabled={loading()}
              onClick={handleOidcLogin}
            >
              {loading()
                ? t("auth.oidc.starting")
                : `${t("auth.oidc.login")} ${oidcStatus()?.provider_name ?? "OIDC"}`}
            </button>
          </Show>
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
