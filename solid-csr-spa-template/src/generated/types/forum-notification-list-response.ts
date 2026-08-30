// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumNotificationCursorResponse } from "./forum-notification-cursor-response";
import type { ForumNotificationResponse } from "./forum-notification-response";

export type ForumNotificationListResponse = {
  readonly next_cursor: ForumNotificationCursorResponse | null;
  readonly notifications: ReadonlyArray<ForumNotificationResponse>;
};
