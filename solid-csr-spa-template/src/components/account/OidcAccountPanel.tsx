import { createSignal, onSettled, Show } from "solid-js";
import type { OidcStatusResponse } from "../../generated";
import { oidcApi } from "../../services/all_api";
import { consumeOidcFragment } from "../../services/oidc_fragment";
import { t } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";

export default function OidcAccountPanel() {
  const [status, setStatus] = createSignal<OidcStatusResponse | null>(null);
  const [currentPassword, setCurrentPassword] = createSignal("");
  const [pending, setPending] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const loadStatus = async () => {
    const response = await oidcApi.status();
    setStatus(response.data ?? null);
  };

  const completeLink = async (completionToken: string) => {
    setPending(true);
    setError(null);
    try {
      const response = await oidcApi.completeLink({
        completion_token: completionToken,
      });
      if (!response.success || !response.data?.linked) {
        throw new Error("link completion rejected");
      }
      setMessage(t("profile.oidc.link_success"));
    } catch {
      setError(t("profile.oidc.failed"));
    } finally {
      await loadStatus().catch(() => setError(t("profile.oidc.failed")));
      setPending(false);
    }
  };

  onSettled(() => {
    const callback = consumeOidcFragment();
    if (callback.kind === "link-ready") {
      void completeLink(callback.completionToken);
      return;
    }
    if (callback.kind === "failed") setError(t("profile.oidc.failed"));
    loadStatus().catch(() => setError(t("profile.oidc.failed")));
  });

  const startLink = async () => {
    setPending(true);
    setMessage(null);
    setError(null);
    try {
      const response = await oidcApi.startLink();
      const authorizationUrl = response.data?.authorization_url;
      if (!response.success || !authorizationUrl) {
        throw new Error("authorization URL missing");
      }
      window.location.assign(authorizationUrl);
    } catch {
      setError(t("profile.oidc.failed"));
      setPending(false);
    }
  };

  const unlink = async (event: Event) => {
    event.preventDefault();
    if (!currentPassword()) {
      setError(t("profile.update.password_required"));
      return;
    }
    setPending(true);
    setMessage(null);
    setError(null);
    try {
      const response = await oidcApi.unlink({
        current_password: currentPassword(),
      });
      if (!response.success || response.data?.linked !== false) {
        throw new Error("unlink rejected");
      }
      setCurrentPassword("");
      setMessage(t("profile.oidc.unlink_success"));
      await loadStatus();
    } catch {
      setError(t("profile.oidc.failed"));
    } finally {
      setPending(false);
    }
  };

  return (
    <Show when={status()?.enabled}>
      <section class={`${pageStyles.card} oidc-account-panel p-6`}>
        <h2 class="text-lg font-semibold">{t("profile.oidc.title")}</h2>
        <p class={`${pageStyles.muted} mt-2`}>
          {t("profile.oidc.description")}
        </p>
        <p class="mt-3">
          {status()?.linked
            ? t("profile.oidc.linked")
            : t("profile.oidc.not_linked")}{" "}
          {status()?.provider_name ?? "OIDC"}
        </p>
        <Show
          when={status()?.linked}
          fallback={
            <button
              type="button"
              class={`${pageStyles.buttonPrimary} mt-4`}
              disabled={pending()}
              onClick={startLink}
            >
              {pending() ? t("profile.oidc.linking") : t("profile.oidc.link")}
            </button>
          }
        >
          <form class="mt-4" onSubmit={unlink}>
            <label class="block mb-2" for="oidc-unlink-password">
              {t("profile.oidc.current_password")}
            </label>
            <input
              id="oidc-unlink-password"
              class={`${pageStyles.input} mb-3`}
              type="password"
              autocomplete="current-password"
              maxlength={128}
              value={currentPassword()}
              onInput={(event) => setCurrentPassword(event.currentTarget.value)}
              required
            />
            <button
              type="submit"
              class={pageStyles.buttonSecondary}
              disabled={pending()}
            >
              {pending()
                ? t("profile.oidc.unlinking")
                : t("profile.oidc.unlink")}
            </button>
          </form>
        </Show>
        <Show when={message()}>
          <p class={`${pageStyles.alertSuccess} mt-4`}>{message()}</p>
        </Show>
        <Show when={error()}>
          <p class={`${pageStyles.alertError} mt-4`}>{error()}</p>
        </Show>
      </section>
    </Show>
  );
}
