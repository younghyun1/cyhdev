// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumTopicCursorResponse } from "./forum-topic-cursor-response";
import type { ForumTopicResponse } from "./forum-topic-response";

export type ForumTopicListResponse = {
  readonly next_cursor: ForumTopicCursorResponse | null;
  readonly topics: ReadonlyArray<ForumTopicResponse>;
};
