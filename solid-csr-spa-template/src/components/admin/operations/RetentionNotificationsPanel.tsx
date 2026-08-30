import { For, Show, createSignal, onCleanup, onSettled } from "solid-js";

import type {
  RetentionNotificationStatusItem,
  RetryRetentionNotificationResponse,
} from "../../../generated";
import type { AdminOperationsApi } from "../../../services/contracts/admin_operations";
import { adminOperationsApi } from "../../../services/contracts/admin_operations";
import { t, tx } from "../../../state/i18n";
import {
  formatAdminTimestamp,
  operationErrorMessage,
  retentionStageKey,
  retentionStatusKey,
} from "./operationsFormat";

const PAGE_SIZE = 25;
const MAX_CLIENT_PAGES = 100;

type Cursor = {
  readonly nextAttemptAt: string;
  readonly notificationId: string;
};

type Props = {
  readonly service?: Pick<
    AdminOperationsApi,
    "retentionStatus" | "retryRetentionNotification"
  >;
};

export default function RetentionNotificationsPanel(props: Props) {
  const service = () => props.service ?? adminOperationsApi;
  const [notifications, setNotifications] = createSignal<
    readonly RetentionNotificationStatusItem[]
  >([]);
  const [nextCursor, setNextCursor] = createSignal<Cursor | null>(null);
  const [page, setPage] = createSignal(1);
  const [loading, setLoading] = createSignal(true);
  const [loadError, setLoadError] = createSignal<string | null>(null);
  const [pendingRetry, setPendingRetry] =
    createSignal<RetentionNotificationStatusItem | null>(null);
  const [confirmed, setConfirmed] = createSignal(false);
  const [retryingId, setRetryingId] = createSignal<string | null>(null);
  const [retryError, setRetryError] = createSignal<string | null>(null);
  const [receipt, setReceipt] =
    createSignal<RetryRetentionNotificationResponse | null>(null);
  let activeLoad: AbortController | null = null;

  onSettled(() => void loadPage(null, 1));
  onCleanup(() => activeLoad?.abort());

  const loadPage = async (cursor: Cursor | null, nextPage: number) => {
    activeLoad?.abort();
    const controller = new AbortController();
    activeLoad = controller;
    setLoading(true);
    setLoadError(null);
    try {
      const response = await service().retentionStatus(
        cursor === null
          ? { limit: PAGE_SIZE }
          : {
              after_next_attempt_at: cursor.nextAttemptAt,
              after_notification_id: cursor.notificationId,
              limit: PAGE_SIZE,
            },
        controller.signal,
      );
      if (controller.signal.aborted) return;
      setNotifications(response.data.notifications);
      setNextCursor(cursorFromResponse(response.data));
      setPage(nextPage);
    } catch (error: unknown) {
      if (!controller.signal.aborted) setLoadError(operationErrorMessage(error));
    } finally {
      if (!controller.signal.aborted) setLoading(false);
    }
  };

  const retry = async () => {
    const notification = pendingRetry();
    if (notification === null || !confirmed() || retryingId() !== null) return;
    setRetryingId(notification.notification_id);
    setRetryError(null);
    try {
      const response = await service().retryRetentionNotification(
        notification.notification_id,
      );
      setReceipt(response.data);
      setPendingRetry(null);
      setConfirmed(false);
      await loadPage(null, 1);
    } catch (error: unknown) {
      setRetryError(operationErrorMessage(error));
      await loadPage(null, 1);
    } finally {
      setRetryingId(null);
    }
  };

  return (
    <section class="operations-panel" aria-labelledby="retention-heading">
      <div class="operations-panel-heading">
        <div>
          <h2 id="retention-heading">{t("operations.retention.title")}</h2>
          <p>{t("operations.retention.description")}</p>
        </div>
        <button type="button" onClick={() => loadPage(null, 1)} disabled={loading()}>
          {t("common.refresh")}
        </button>
      </div>
      <Show when={loadError() !== null}>
        <p class="operations-error" role="alert">{loadError()}</p>
      </Show>
      <Show when={receipt()}>
        {(value) => (
          <p class="operations-receipt" role="status">
            {tx("operations.retention.retry_success", {
              notification_id: value().notification_id,
              next_attempt_at: formatAdminTimestamp(value().next_attempt_at),
            })}
          </p>
        )}
      </Show>
      <Show when={!loading() || notifications().length > 0} fallback={
        <p class="operations-muted">{t("common.loading")}</p>
      }>
        <div class="operations-table-wrap">
          <table class="operations-table">
            <thead>
              <tr>
                <th>{t("operations.retention.notification")}</th>
                <th>{t("operations.retention.schedule")}</th>
                <th>{t("operations.retention.status")}</th>
                <th>{t("authorization.action")}</th>
              </tr>
            </thead>
            <tbody>
              <For each={notifications()}>{(item) => (
                <tr>
                  <td>
                    <strong>{t(retentionStageKey(item.stage))}</strong>
                    <small>{item.notification_id}</small>
                    <small>{tx("operations.retention.user", { user_id: item.user_id })}</small>
                  </td>
                  <td>
                    {formatAdminTimestamp(item.scheduled_for)}
                    <small>{tx("operations.retention.next_attempt", {
                      timestamp: formatAdminTimestamp(item.next_attempt_at),
                    })}</small>
                    <small>{tx("operations.retention.attempts", {
                      count: item.attempt_count,
                    })}</small>
                  </td>
                  <td>
                    <span class="operations-status">{t(retentionStatusKey(item))}</span>
                    <Show when={item.last_error}>{(message) => (
                      <small class="operations-failure-detail">{message()}</small>
                    )}</Show>
                  </td>
                  <td>
                    <button
                      type="button"
                      disabled={item.sent_at != null || item.cancelled_at != null || retryingId() !== null}
                      onClick={() => {
                        setRetryError(null);
                        setConfirmed(false);
                        setPendingRetry(item);
                      }}
                    >
                      {t("operations.retention.retry")}
                    </button>
                  </td>
                </tr>
              )}</For>
            </tbody>
          </table>
        </div>
        <Show when={notifications().length === 0}>
          <p class="operations-muted">{t("operations.retention.empty")}</p>
        </Show>
        <div class="operations-pagination">
          <span>{tx("operations.retention.page", { page: page() })}</span>
          <button
            type="button"
            disabled={
              loading() ||
              notifications().length !== PAGE_SIZE ||
              nextCursor() === null ||
              page() >= MAX_CLIENT_PAGES
            }
            onClick={() => {
              const cursor = nextCursor();
              if (cursor !== null) void loadPage(cursor, page() + 1);
            }}
          >
            {t("common.next")}
          </button>
        </div>
      </Show>
      <Show when={pendingRetry()}>{(item) => (
        <div class="operations-dialog-backdrop" role="presentation">
          <section class="operations-dialog" role="dialog" aria-modal="true" aria-labelledby="retry-title">
            <h3 id="retry-title">{t("operations.retention.retry_confirm_title")}</h3>
            <p>{tx("operations.retention.retry_confirm_description", {
              notification_id: item().notification_id,
              user_id: item().user_id,
              stage: t(retentionStageKey(item().stage)),
            })}</p>
            <label class="operations-confirm-checkbox">
              <input type="checkbox" checked={confirmed()} onChange={(event) => setConfirmed(event.currentTarget.checked)} />
              <span>{t("operations.retention.retry_confirm_checkbox")}</span>
            </label>
            <Show when={retryError() !== null}>
              <p class="operations-error" role="alert">{retryError()}</p>
            </Show>
            <div class="operations-dialog-actions">
              <button type="button" disabled={retryingId() !== null} onClick={() => setPendingRetry(null)}>{t("common.cancel")}</button>
              <button type="button" class="operations-primary-button" disabled={!confirmed() || retryingId() !== null} onClick={() => void retry()}>
                {retryingId() !== null ? t("operations.retention.retrying") : t("operations.retention.retry")}
              </button>
            </div>
          </section>
        </div>
      )}</Show>
    </section>
  );
}

function cursorFromResponse(data: {
  readonly next_after_next_attempt_at?: string | null;
  readonly next_after_notification_id?: string | null;
}): Cursor | null {
  return data.next_after_next_attempt_at != null && data.next_after_notification_id != null
    ? {
        nextAttemptAt: data.next_after_next_attempt_at,
        notificationId: data.next_after_notification_id,
      }
    : null;
}
