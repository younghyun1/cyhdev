import { For, Show, createSignal } from "solid-js";

import type { HardPurgeAccountResponse } from "../../../generated";
import type { AdminOperationsApi } from "../../../services/contracts/admin_operations";
import { adminOperationsApi } from "../../../services/contracts/admin_operations";
import { t, tx } from "../../../state/i18n";
import {
  formatAdminTimestamp,
  isUuid,
  operationErrorMessage,
} from "./operationsFormat";

type Props = {
  readonly service?: Pick<AdminOperationsApi, "hardPurgeAccount">;
};

type PurgeReceipt = Omit<
  HardPurgeAccountResponse,
  "profile_cleanup_failures"
> & {
  readonly profile_cleanup_failures: ReadonlyArray<{
    readonly profile_picture_id: string;
    readonly reason: string;
    readonly retryable: boolean;
  }>;
};

export default function HardPurgePanel(props: Props) {
  const service = () => props.service ?? adminOperationsApi;
  const [userId, setUserId] = createSignal("");
  const [confirmation, setConfirmation] = createSignal("");
  const [acknowledged, setAcknowledged] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [receipt, setReceipt] = createSignal<PurgeReceipt | null>(null);

  const normalizedUserId = () => userId().trim().toLowerCase();
  const expectedConfirmation = () => `PURGE ${normalizedUserId()}`;
  const valid = () =>
    isUuid(normalizedUserId()) &&
    confirmation() === expectedConfirmation() &&
    acknowledged();

  const purge = async (event: SubmitEvent) => {
    event.preventDefault();
    const target = normalizedUserId();
    if (!valid() || busy()) return;
    setBusy(true);
    setError(null);
    setReceipt(null);
    try {
      const response = await service().hardPurgeAccount(target);
      setReceipt(sanitizeReceipt(response.data));
      setConfirmation("");
      setAcknowledged(false);
    } catch (mutationError: unknown) {
      setError(operationErrorMessage(mutationError));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section class="operations-panel operations-danger-panel" aria-labelledby="purge-heading">
      <div class="operations-panel-heading">
        <div>
          <h2 id="purge-heading">{t("operations.purge.title")}</h2>
          <p>{t("operations.purge.description")}</p>
        </div>
      </div>
      <p class="operations-danger-note">{t("operations.purge.warning")}</p>
      <form class="operations-purge-form" onSubmit={purge}>
        <label for="hard-purge-user-id">{t("operations.purge.user_id")}</label>
        <input
          id="hard-purge-user-id"
          value={userId()}
          inputmode="text"
          autocomplete="off"
          spellcheck={false}
          maxlength={36}
          required
          aria-invalid={
            userId().trim() !== "" && !isUuid(normalizedUserId())
              ? "true"
              : "false"
          }
          onInput={(event) => {
            setUserId(event.currentTarget.value);
            setConfirmation("");
            setAcknowledged(false);
            setError(null);
          }}
        />
        <Show when={userId().trim() !== "" && !isUuid(normalizedUserId())}>
          <p class="operations-field-error">{t("operations.purge.invalid_uuid")}</p>
        </Show>
        <label for="hard-purge-confirmation">
          {tx("operations.purge.confirmation", {
            phrase: expectedConfirmation(),
          })}
        </label>
        <input
          id="hard-purge-confirmation"
          value={confirmation()}
          autocomplete="off"
          spellcheck={false}
          maxlength={42}
          required
          onInput={(event) => setConfirmation(event.currentTarget.value)}
        />
        <label class="operations-confirm-checkbox">
          <input
            type="checkbox"
            checked={acknowledged()}
            onChange={(event) => setAcknowledged(event.currentTarget.checked)}
          />
          <span>{t("operations.purge.acknowledgement")}</span>
        </label>
        <Show when={error() !== null}>
          <p class="operations-error" role="alert">{error()}</p>
        </Show>
        <button
          type="submit"
          class="operations-danger-button"
          disabled={!valid() || busy()}
        >
          {busy() ? t("operations.purge.purging") : t("operations.purge.submit")}
        </button>
      </form>
      <Show when={receipt()}>{(value) => (
        <section class="operations-purge-receipt" role="status" aria-labelledby="purge-receipt-heading">
          <h3 id="purge-receipt-heading">{t("operations.purge.receipt_title")}</h3>
          <p class={value().profile_cleanup_remaining === 0 ? "operations-receipt" : "operations-warning-receipt"}>
            {value().profile_cleanup_remaining === 0
              ? t("operations.purge.receipt_complete")
              : t("operations.purge.receipt_partial")}
          </p>
          <dl class="operations-details operations-receipt-details">
            <div>
              <dt>{t("operations.purge.user_id")}</dt>
              <dd>{value().user_id}</dd>
            </div>
            <div>
              <dt>{t("operations.purge.purged_at")}</dt>
              <dd>{formatAdminTimestamp(value().hard_purged_at)}</dd>
            </div>
            <div>
              <dt>{t("operations.purge.objects_deleted")}</dt>
              <dd>{value().profile_objects_deleted}</dd>
            </div>
            <div>
              <dt>{t("operations.purge.metadata_deleted")}</dt>
              <dd>{value().profile_metadata_deleted}</dd>
            </div>
            <div>
              <dt>{t("operations.purge.cleanup_remaining")}</dt>
              <dd>{value().profile_cleanup_remaining}</dd>
            </div>
          </dl>
          <Show when={value().profile_cleanup_failures.length > 0}>
            <h4>{t("operations.purge.failures")}</h4>
            <ul class="operations-failure-list">
              <For each={value().profile_cleanup_failures}>{(failure) => (
                <li>
                  <strong>{failure.profile_picture_id}</strong>
                  <span>{failure.reason}</span>
                  <small>
                    {failure.retryable
                      ? t("operations.purge.retryable")
                      : t("operations.purge.not_retryable")}
                  </small>
                </li>
              )}</For>
            </ul>
          </Show>
        </section>
      )}</Show>
    </section>
  );
}

function sanitizeReceipt(receipt: HardPurgeAccountResponse): PurgeReceipt {
  return {
    user_id: receipt.user_id,
    hard_purged_at: receipt.hard_purged_at,
    profile_objects_deleted: receipt.profile_objects_deleted,
    profile_metadata_deleted: receipt.profile_metadata_deleted,
    profile_cleanup_remaining: receipt.profile_cleanup_remaining,
    profile_cleanup_failures: receipt.profile_cleanup_failures.map((failure) => ({
      profile_picture_id: failure.profile_picture_id,
      reason: failure.reason,
      retryable: failure.retryable,
    })),
  };
}
