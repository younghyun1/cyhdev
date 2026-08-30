// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumAuthorResponse } from "./forum-author-response";
import type { ForumContentStateResponse } from "./forum-content-state-response";
import type { ForumTopicAccessStateResponse } from "./forum-topic-access-state-response";

export type ForumTopicResponse = {
  readonly access_state: ForumTopicAccessStateResponse;
  readonly author: ForumAuthorResponse;
  readonly body: string | null;
  readonly content_state: ForumContentStateResponse;
  readonly created_at: string;
  readonly edited_at: string | null;
  readonly is_pinned: boolean;
  readonly last_activity_at: string;
  readonly reply_count: number;
  readonly revision: number;
  readonly title: string | null;
  readonly topic_id: string;
  readonly updated_at: string;
};
