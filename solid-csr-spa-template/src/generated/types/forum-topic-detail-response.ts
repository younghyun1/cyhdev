// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumReplyCursorResponse } from "./forum-reply-cursor-response";
import type { ForumReplyResponse } from "./forum-reply-response";
import type { ForumTopicResponse } from "./forum-topic-response";

export type ForumTopicDetailResponse = {
  readonly is_subscribed: boolean;
  readonly next_reply_cursor: ForumReplyCursorResponse | null;
  readonly replies: ReadonlyArray<ForumReplyResponse>;
  readonly topic: ForumTopicResponse;
};
