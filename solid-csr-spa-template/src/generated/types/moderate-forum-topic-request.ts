// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumTopicModerationActionRequest } from "./forum-topic-moderation-action-request";

export type ModerateForumTopicRequest = {
  readonly action: ForumTopicModerationActionRequest;
  readonly expected_revision: number;
  readonly reason: string;
};
