import { Show, createSignal, onCleanup } from "solid-js";
import { useNavigate } from "@solidjs/router";
import type { DeleteAccountResponse } from "../../generated";
import { authApi } from "../../services/all_api";
import {
  setAuthenticated,
  setSuperuser,
  setUser,
} from "../../state/auth";
import { t, tx } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";

const REDIRECT_DELAY_MS = 5_000;

function formatDeadline(value: string | undefined): string {
  if (value === undefined) return t("common.unknown_date");
  const deadline = new Date(value);
  if (Number.isNaN(deadline.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "long",
    timeStyle: "long",
  }).format(deadline);
}

export default function AccountDeletionPanel() {
  const navigate = useNavigate();
  const [currentPassword, setCurrentPassword] = createSignal("");
  const [confirmed, setConfirmed] = createSignal(false);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [receipt, setReceipt] = createSignal<DeleteAccountResponse | null>(null);
  let redirectTimer: ReturnType<typeof setTimeout> | undefined;

  const clearAuthAndLeave = () => {
    setUser(null);
    setSuperuser(false);
    setAuthenticated(false);
    navigate("/", { replace: true });
  };

  onCleanup(() => {
    if (redirectTimer !== undefined) clearTimeout(redirectTimer);
  });

  const handleDelete = async (event: SubmitEvent) => {
    event.preventDefault();
    if (pending() || !confirmed() || currentPassword().length === 0) return;

    setPending(true);
    setError(null);
    try {
      const response = await authApi.deleteAccount({
        current_password: currentPassword(),
      });
      if (!response.success) {
        setError(t("profile.delete_account.failed"));
        return;
      }
      setReceipt(response.data);
      setConfirmed(false);
      setUser(null);
      setSuperuser(false);
      redirectTimer = setTimeout(clearAuthAndLeave, REDIRECT_DELAY_MS);
    } catch (caught: unknown) {
      setError(
        caught instanceof Error
          ? caught.message
          : t("profile.delete_account.failed"),
      );
    } finally {
      setCurrentPassword("");
      setPending(false);
    }
  };

  return (
    <section class={`${pageStyles.cardPadded} account-deletion-panel border-danger/40`}>
      <h2 class="text-lg font-semibold text-danger">
        {t("profile.delete_account.title")}
      </h2>
      <hr class={`my-3 ${pageStyles.divider}`} />
      <Show
        when={receipt() === null}
        fallback={
          <div class="space-y-3" aria-live="polite">
            <div class={pageStyles.alertSuccess}>
              <p class="font-semibold">
                {t("profile.delete_account.success")}
              </p>
              <p class="mt-1">
                {tx("profile.delete_account.purge_deadline", {
                  deadline: formatDeadline(receipt()?.purge_after),
                })}
              </p>
            </div>
            <p class={pageStyles.muted}>
              {t("profile.delete_account.redirecting")}
            </p>
            <button
              type="button"
              class={pageStyles.buttonSecondary}
              onClick={clearAuthAndLeave}
            >
              {t("common.go_home")}
            </button>
          </div>
        }
      >
        <form class="space-y-4" onSubmit={handleDelete}>
          <p class={pageStyles.muted}>
            {t("profile.delete_account.description")}
          </p>
          <p class={pageStyles.muted} id="account-deletion-retention">
            {t("profile.delete_account.retention")}
          </p>
          <label class="block space-y-2">
            <span class="text-sm font-medium text-ink">
              {t("profile.delete_account.current_password")}
            </span>
            <input
              class={pageStyles.input}
              type="password"
              autocomplete="current-password"
              value={currentPassword()}
              onInput={(event) => setCurrentPassword(event.currentTarget.value)}
              disabled={pending()}
              required
            />
          </label>
          <label class="flex items-start gap-3 text-sm text-ink">
            <input
              class="mt-1 h-4 w-4 shrink-0"
              type="checkbox"
              checked={confirmed()}
              onChange={(event) => setConfirmed(event.currentTarget.checked)}
              aria-describedby="account-deletion-retention"
              disabled={pending()}
              required
            />
            <span>{t("profile.delete_account.confirmation")}</span>
          </label>
          <Show when={error()}>
            <div class={pageStyles.alertError} role="alert">
              {error()}
            </div>
          </Show>
          <button
            class={pageStyles.buttonDanger}
            type="submit"
            disabled={
              pending() || !confirmed() || currentPassword().length === 0
            }
          >
            {pending()
              ? t("common.deleting")
              : t("profile.delete_account.submit")}
          </button>
        </form>
      </Show>
    </section>
  );
}
