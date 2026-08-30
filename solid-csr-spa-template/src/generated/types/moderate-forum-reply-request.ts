// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumReplyModerationActionRequest } from "./forum-reply-moderation-action-request";

export type ModerateForumReplyRequest = {
  readonly action: ForumReplyModerationActionRequest;
  readonly expected_revision: number;
  readonly reason: string;
};
