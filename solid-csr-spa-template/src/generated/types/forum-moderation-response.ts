// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumModerationActionResponse } from "./forum-moderation-action-response";

export type ForumModerationResponse = {
  readonly action: ForumModerationActionResponse;
  readonly audit_event_id: string;
  readonly moderated_at: string;
  readonly revision: number;
  readonly target_id: string;
};
