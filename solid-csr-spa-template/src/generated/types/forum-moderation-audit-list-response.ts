// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumModerationAuditCursorResponse } from "./forum-moderation-audit-cursor-response";
import type { ForumModerationAuditItem } from "./forum-moderation-audit-item";

export type ForumModerationAuditListResponse = {
  readonly events: ReadonlyArray<ForumModerationAuditItem>;
  readonly next_cursor: ForumModerationAuditCursorResponse | null;
};
