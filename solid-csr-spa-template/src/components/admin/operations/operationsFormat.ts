import type { RetentionNotificationStatusItem } from "../../../generated";
import type { UiTextKey } from "../../../i18n/keys";
import { locale, t } from "../../../state/i18n";

const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function isUuid(value: string): boolean {
  return UUID_PATTERN.test(value);
}

export function formatAdminTimestamp(value: string | null | undefined): string {
  if (value === null || value === undefined) return t("common.n_a");
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return t("common.unknown_date");
  return new Intl.DateTimeFormat(locale(), {
    dateStyle: "medium",
    timeStyle: "medium",
    timeZone: "UTC",
  }).format(parsed);
}

export function retentionStageKey(
  stage: RetentionNotificationStatusItem["stage"],
): UiTextKey {
  return stage === "seven_days_before_purge"
    ? "operations.retention.stage.seven_days"
    : "operations.retention.stage.one_day";
}

export function retentionStatusKey(
  notification: RetentionNotificationStatusItem,
): UiTextKey {
  if (notification.cancelled_at !== null && notification.cancelled_at !== undefined) {
    return "operations.retention.status.cancelled";
  }
  if (notification.sent_at !== null && notification.sent_at !== undefined) {
    return "operations.retention.status.sent";
  }
  if (
    notification.claim_expires_at !== null &&
    notification.claim_expires_at !== undefined
  ) {
    return "operations.retention.status.processing";
  }
  if (notification.last_error !== null && notification.last_error !== undefined) {
    return "operations.retention.status.failed";
  }
  return "operations.retention.status.pending";
}

export function operationErrorMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : t("operations.error.unknown");
}
