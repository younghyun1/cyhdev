import { For, Show, createSignal, onCleanup, onSettled } from "solid-js";

import type { UnresolvedMediaCleanupItem } from "../../../generated";
import type { AdminOperationsApi } from "../../../services/contracts/admin_operations";
import {
  MEDIA_CLEANUP_BUCKET,
  adminOperationsApi,
} from "../../../services/contracts/admin_operations";
import { t, tx } from "../../../state/i18n";
import { formatAdminTimestamp, operationErrorMessage } from "./operationsFormat";

type Props = {
  readonly service?: Pick<
    AdminOperationsApi,
    "unresolvedMediaCleanup" | "resolveMediaCleanup"
  >;
};

export default function MediaCleanupPanel(props: Props) {
  const service = () => props.service ?? adminOperationsApi;
  const [records, setRecords] = createSignal<
    readonly UnresolvedMediaCleanupItem[]
  >([]);
  const [loading, setLoading] = createSignal(true);
  const [loadError, setLoadError] = createSignal<string | null>(null);
  const [pending, setPending] =
    createSignal<UnresolvedMediaCleanupItem | null>(null);
  const [objectKey, setObjectKey] = createSignal("");
  const [confirmed, setConfirmed] = createSignal(false);
  const [resolvingId, setResolvingId] = createSignal<string | null>(null);
  const [mutationError, setMutationError] = createSignal<string | null>(null);
  const [receiptId, setReceiptId] = createSignal<string | null>(null);
  let activeLoad: AbortController | null = null;

  onSettled(() => void loadRecords());
  onCleanup(() => activeLoad?.abort());

  const loadRecords = async () => {
    activeLoad?.abort();
    const controller = new AbortController();
    activeLoad = controller;
    setLoading(true);
    setLoadError(null);
    try {
      const response = await service().unresolvedMediaCleanup(controller.signal);
      if (!controller.signal.aborted) setRecords(response.data.records);
    } catch (error: unknown) {
      if (!controller.signal.aborted) setLoadError(operationErrorMessage(error));
    } finally {
      if (!controller.signal.aborted) setLoading(false);
    }
  };

  const openReconciliation = (record: UnresolvedMediaCleanupItem) => {
    setPending(record);
    setObjectKey("");
    setConfirmed(false);
    setMutationError(null);
  };

  const resolve = async (event: SubmitEvent) => {
    event.preventDefault();
    const record = pending();
    const nextKey = objectKey();
    if (
      record === null ||
      resolvingId() !== null ||
      !confirmed() ||
      nextKey.length === 0
    ) return;

    setResolvingId(record.cleanup_id);
    setMutationError(null);
    try {
      const response = await service().resolveMediaCleanup(record.cleanup_id, {
        expected_original_url: record.original_url,
        bucket: MEDIA_CLEANUP_BUCKET,
        key: nextKey,
      });
      setReceiptId(response.data.cleanup_id);
      setRecords((current) =>
        current.filter((item) => item.cleanup_id !== record.cleanup_id),
      );
      closeDialog();
      await loadRecords();
    } catch (error: unknown) {
      setMutationError(operationErrorMessage(error));
      await loadRecords();
    } finally {
      setResolvingId(null);
    }
  };

  const closeDialog = () => {
    setPending(null);
    setObjectKey("");
    setConfirmed(false);
  };

  return (
    <section class="operations-panel" aria-labelledby="media-cleanup-heading">
      <div class="operations-panel-heading">
        <div>
          <h2 id="media-cleanup-heading">{t("operations.media.title")}</h2>
          <p>{t("operations.media.description")}</p>
          <Show when={records().length === 100}>
            <p class="operations-bound-note">{t("operations.media.bound_note")}</p>
          </Show>
        </div>
        <button type="button" onClick={() => loadRecords()} disabled={loading()}>
          {t("common.refresh")}
        </button>
      </div>
      <Show when={loadError() !== null}>
        <p class="operations-error" role="alert">{loadError()}</p>
      </Show>
      <Show when={receiptId()}>{(cleanupId) => (
        <p class="operations-receipt" role="status">
          {tx("operations.media.success", { cleanup_id: cleanupId() })}
        </p>
      )}</Show>
      <Show when={!loading() || records().length > 0} fallback={
        <p class="operations-muted">{t("common.loading")}</p>
      }>
        <div class="operations-record-list">
          <For each={records()}>{(record) => (
            <article class="operations-record">
              <div class="operations-record-heading">
                <div>
                  <strong>{record.cleanup_id}</strong>
                  <small>{tx("operations.media.created", {
                    timestamp: formatAdminTimestamp(record.created_at),
                  })}</small>
                </div>
                <button
                  type="button"
                  disabled={resolvingId() !== null}
                  onClick={() => openReconciliation(record)}
                >
                  {t("operations.media.reconcile")}
                </button>
              </div>
              <dl class="operations-details">
                <div>
                  <dt>{t("operations.media.source_id")}</dt>
                  <dd>{record.source_id}</dd>
                </div>
                <div>
                  <dt>{t("operations.media.reason")}</dt>
                  <dd>{record.reason}</dd>
                </div>
                <div>
                  <dt>{t("operations.media.original_url")}</dt>
                  <dd><code class="operations-inert-url">{record.original_url}</code></dd>
                </div>
              </dl>
            </article>
          )}</For>
        </div>
        <Show when={records().length === 0}>
          <p class="operations-muted">{t("operations.media.empty")}</p>
        </Show>
      </Show>
      <Show when={pending()}>{(record) => (
        <div class="operations-dialog-backdrop" role="presentation">
          <section class="operations-dialog" role="dialog" aria-modal="true" aria-labelledby="media-dialog-title">
            <h3 id="media-dialog-title">{t("operations.media.dialog_title")}</h3>
            <p>{t("operations.media.dialog_description")}</p>
            <dl class="operations-details operations-dialog-resource">
              <div>
                <dt>{t("operations.media.cleanup_id")}</dt>
                <dd>{record().cleanup_id}</dd>
              </div>
              <div>
                <dt>{t("operations.media.original_url")}</dt>
                <dd><code class="operations-inert-url">{record().original_url}</code></dd>
              </div>
            </dl>
            <form onSubmit={resolve}>
              <label for="cleanup-bucket">{t("operations.media.bucket")}</label>
              <input
                id="cleanup-bucket"
                value={MEDIA_CLEANUP_BUCKET}
                readonly
              />
              <label for="cleanup-key">{t("operations.media.key")}</label>
              <input
                id="cleanup-key"
                value={objectKey()}
                maxlength={1024}
                autocomplete="off"
                required
                onInput={(event) => setObjectKey(event.currentTarget.value)}
              />
              <label class="operations-confirm-checkbox">
                <input type="checkbox" checked={confirmed()} onChange={(event) => setConfirmed(event.currentTarget.checked)} />
                <span>{t("operations.media.confirm")}</span>
              </label>
              <Show when={mutationError() !== null}>
                <p class="operations-error" role="alert">{mutationError()}</p>
              </Show>
              <div class="operations-dialog-actions">
                <button type="button" disabled={resolvingId() !== null} onClick={closeDialog}>{t("common.cancel")}</button>
                <button
                  type="submit"
                  class="operations-primary-button"
                  disabled={!confirmed() || objectKey().length === 0 || resolvingId() !== null}
                >
                  {resolvingId() !== null ? t("operations.media.reconciling") : t("operations.media.reconcile")}
                </button>
              </div>
            </form>
          </section>
        </div>
      )}</Show>
    </section>
  );
}
