// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { RetentionNotificationStage } from "./retention-notification-stage";

export type RetentionNotificationStatusItem = {
  readonly attempt_count: number;
  readonly cancelled_at?: string | null;
  readonly claim_expires_at?: string | null;
  readonly last_error?: string | null;
  readonly next_attempt_at: string;
  readonly notification_id: string;
  readonly scheduled_for: string;
  readonly sent_at?: string | null;
  readonly stage: RetentionNotificationStage;
  readonly user_id: string;
};
