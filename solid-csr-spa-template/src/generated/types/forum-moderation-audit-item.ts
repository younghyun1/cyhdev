// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumAuthorResponse } from "./forum-author-response";
import type { ForumModerationActionResponse } from "./forum-moderation-action-response";

export type ForumModerationAuditItem = {
  readonly action: ForumModerationActionResponse;
  readonly actor: ForumAuthorResponse;
  readonly audit_event_id: string;
  readonly created_at: string;
  readonly reason: string;
  readonly reply_id: string | null;
  readonly request_id: string | null;
  readonly topic_id: string | null;
};
