// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { RetentionNotificationStatusItem } from "./retention-notification-status-item";

export type RetentionNotificationStatusResponse = {
  readonly next_after_next_attempt_at?: string | null;
  readonly next_after_notification_id?: string | null;
  readonly notifications: ReadonlyArray<RetentionNotificationStatusItem>;
};
