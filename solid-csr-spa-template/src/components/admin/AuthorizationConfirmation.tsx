import { createEffect, createSignal, Show } from "solid-js";

import { t } from "../../state/i18n";
import type { PendingAuthorizationChange } from "./authorizationTypes";

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

  const description = () => {
    const change = props.change;
    if (change === null) return "";
    if (change.kind === "role") {
      return `${change.user.user_name}: ${change.user.role_name} → ${change.role.role_name}`;
    }
    const action = change.enabled
      ? t("authorization.permission.grant")
      : t("authorization.permission.revoke");
    return `${action}: ${change.role.role_name} / ${change.permission.permission_name}`;
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
      <div class="authorization-dialog-backdrop" role="presentation">
        <section
          class="authorization-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="authorization-confirm-title"
        >
          <h2 id="authorization-confirm-title">
            {t("authorization.confirm.title")}
          </h2>
          <p class="authorization-dialog-change">{description()}</p>
          <form onSubmit={submit}>
            <label for="authorization-reason">
              {t("authorization.confirm.reason")}
            </label>
            <textarea
              id="authorization-reason"
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
              <button type="button" onClick={props.onCancel} disabled={props.busy}>
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
        </section>
      </div>
    </Show>
  );
}
