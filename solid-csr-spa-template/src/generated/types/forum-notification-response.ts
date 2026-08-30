// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumAuthorResponse } from "./forum-author-response";
import type { ForumNotificationKindResponse } from "./forum-notification-kind-response";

export type ForumNotificationResponse = {
  readonly actor: ForumAuthorResponse;
  readonly created_at: string;
  readonly expires_at: string;
  readonly kind: ForumNotificationKindResponse;
  readonly notification_id: string;
  readonly read_at: string | null;
  readonly reply_id: string;
  readonly topic_id: string;
  readonly topic_title: string | null;
};
