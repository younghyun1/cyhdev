import { createEffect, createSignal, Show } from "solid-js";

import { t } from "../../state/i18n";
import type { PendingAuthorizationChange } from "./authorizationTypes";
import { MobileDialog } from "../MobileDialog";

type Props = {
  readonly change: PendingAuthorizationChange | null;
  readonly busy: boolean;
  readonly error: string | null;
  readonly onCancel: () => void;
  readonly onConfirm: (reason: string) => Promise<void>;
};

export default function AuthorizationConfirmation(props: Props) {
  const [reason, setReason] = createSignal("");
  const [confirmed, setConfirmed] = createSignal(false);

  createEffect(
    () => props.change,
    () => {
      setReason("");
      setConfirmed(false);
    },
  );

  const roleChange = () => {
    const change = props.change;
    return change !== null && change.kind === "role" ? change : null;
  };

  /** Permission grants and revocations; names are identifiers, so they stay left to right. */
  const permissionDescription = () => {
    const change = props.change;
    if (change === null || change.kind === "role") return null;
    const action = change.enabled
      ? t("authorization.permission.grant")
      : t("authorization.permission.revoke");
    return (
      <>
        {action}: <bdi dir="ltr">{change.role.role_name} / {change.permission.permission_name}</bdi>
      </>
    );
  };

  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (!confirmed() || reason().trim().length < 8 || props.busy) return;
    await props.onConfirm(reason().trim());
    setReason("");
    setConfirmed(false);
  };

  return (
    <Show when={props.change !== null}>
      <MobileDialog
        onClose={() => props.onCancel()}
        overlayClass="authorization-dialog-backdrop"
        panelClass="authorization-dialog"
        ariaLabelledBy="authorization-confirm-title"
        initialFocusSelector="#authorization-reason"
      >
          <h2 id="authorization-confirm-title">
            {t("authorization.confirm.title")}
          </h2>
          <p class="authorization-dialog-change">
            <Show when={roleChange()} fallback={permissionDescription()}>
              {(change) => (
                <>
                  <bdi dir="auto">{change().user.user_name}</bdi>:{" "}
                  <bdi dir="auto">{change().user.role_name}</bdi>{" "}
                  <span class="authorization-change-arrow">→</span>{" "}
                  <bdi dir="auto">{change().role.role_name}</bdi>
                </>
              )}
            </Show>
          </p>
          <form onSubmit={submit}>
            <label for="authorization-reason">
              {t("authorization.confirm.reason")}
            </label>
            <textarea
              id="authorization-reason"
              dir="auto"
              value={reason()}
              maxlength={500}
              minlength={8}
              required
              onInput={(event) => setReason(event.currentTarget.value)}
            />
            <label class="authorization-confirm-checkbox">
              <input
                type="checkbox"
                checked={confirmed()}
                onChange={(event) => setConfirmed(event.currentTarget.checked)}
              />
              <span>{t("authorization.confirm.checkbox")}</span>
            </label>
            <Show when={props.error !== null}>
              <p class="authorization-error" role="alert">
                {props.error}
              </p>
            </Show>
            <div class="authorization-dialog-actions">
              <button
                type="button"
                onClick={() => props.onCancel()}
                disabled={props.busy}
              >
                {t("common.cancel")}
              </button>
              <button
                type="submit"
                class="authorization-primary-button"
                disabled={
                  props.busy || !confirmed() || reason().trim().length < 8
                }
              >
                {props.busy
                  ? t("common.saving")
                  : t("authorization.confirm.apply")}
              </button>
            </div>
          </form>
      </MobileDialog>
    </Show>
  );
}
