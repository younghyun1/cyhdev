import { Match, Show, Switch, createSignal, onMount } from "solid-js";
import { useNavigate } from "@solidjs/router";

import type { VerifyUserEmailResponse } from "../generated";
import { authApi } from "../services/all_api";
import { t, tx } from "../state/i18n";
import { pageStyles } from "../styles/pageStyles";
import {
  consumeEmailVerificationFragment,
  type EmailVerificationLinkState,
} from "./verify_email_link";

type LinkState =
  | { readonly kind: "checking" }
  | EmailVerificationLinkState
  | { readonly kind: "confirmed" };
type ConfirmationState = "idle" | "pending" | "success" | "error";

function VerifyEmailPage() {
  const navigate = useNavigate();
  const [linkState, setLinkState] = createSignal<LinkState>({
    kind: "checking",
  });
  const [confirmationState, setConfirmationState] =
    createSignal<ConfirmationState>("idle");
  const [result, setResult] =
    createSignal<VerifyUserEmailResponse | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  onMount(() => {
    setLinkState(
      consumeEmailVerificationFragment(window.location, window.history),
    );
  });

  const confirm = async (): Promise<void> => {
    const link = linkState();
    if (link.kind !== "ready" || confirmationState() === "pending") return;

    setConfirmationState("pending");
    setError(null);
    try {
      const response = await authApi.verifyUserEmail({
        email_validation_token_id: link.token,
      });
      if (!response.success || !response.data) {
        setError(t("auth.verify_email.failed"));
        setConfirmationState("error");
        return;
      }
      setResult(response.data);
      setLinkState({ kind: "confirmed" });
      setConfirmationState("success");
    } catch (cause: unknown) {
      setError(
        cause instanceof Error
          ? cause.message
          : t("auth.verify_email.failed"),
      );
      setConfirmationState("error");
    }
  };

  return (
    <div
      class={`${pageStyles.page} flex items-center justify-center px-6 py-10`}
    >
      <section class={`${pageStyles.card} w-full max-w-md p-8`}>
        <h1 class={`${pageStyles.titleSm} mb-4`}>
          {t("page.verify_email.title")}
        </h1>
        <p class={`${pageStyles.muted} mb-6`}>
          {t("auth.verify_email.no_automatic_change")}
        </p>

        <Switch>
          <Match when={linkState().kind === "checking"}>
            <div class={`${pageStyles.cardPadded} mb-4`} aria-live="polite">
              {t("auth.verify_email.checking")}
            </div>
          </Match>
          <Match when={linkState().kind === "ready"}>
            <div class={`${pageStyles.cardPadded} mb-4`} aria-live="polite">
              {t("auth.verify_email.token_ready")}
            </div>
          </Match>
          <Match when={linkState().kind === "missing"}>
            <div class={`${pageStyles.alertError} mb-4`} role="alert">
              {t("auth.verify_email.token_missing")}
            </div>
          </Match>
          <Match when={linkState().kind === "invalid"}>
            <div class={`${pageStyles.alertError} mb-4`} role="alert">
              {t("auth.verify_email.token_invalid")}
            </div>
          </Match>
          <Match when={linkState().kind === "confirmed"}>
            <div
              class={`${pageStyles.alertSuccess} mb-4`}
              role="status"
              aria-live="polite"
            >
              {t("auth.verify_email.success")}
            </div>
          </Match>
        </Switch>

        <Show when={confirmationState() === "pending"}>
          <p class={`${pageStyles.muted} mb-4`} role="status" aria-live="polite">
            {t("auth.verify_email.confirming")}
          </p>
        </Show>
        <Show when={error()}>
          {(message) => (
            <div class={`${pageStyles.alertError} mb-4`} role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={result()}>
          {(verified) => (
            <p class={`${pageStyles.muted} mb-4`}>
              {tx("auth.verify_email.verified_at", {
                date: new Date(verified().verified_at).toLocaleString(),
              })}
            </p>
          )}
        </Show>

        <Show when={linkState().kind === "ready"}>
          <button
            type="button"
            class={`${pageStyles.buttonPrimary} w-full mb-3 py-3`}
            disabled={confirmationState() === "pending"}
            onClick={() => void confirm()}
          >
            {confirmationState() === "pending"
              ? t("auth.verify_email.confirming")
              : t("auth.verify_email.confirm")}
          </button>
        </Show>
        <button
          type="button"
          class={`${pageStyles.buttonSecondary} w-full py-3`}
          onClick={() => navigate("/login")}
        >
          {t("auth.verify_email.go_to_login")}
        </button>
      </section>
    </div>
  );
}

export default VerifyEmailPage;
