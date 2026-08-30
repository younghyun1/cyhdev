// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ForumAuthorResponse } from "./forum-author-response";
import type { ForumContentStateResponse } from "./forum-content-state-response";

export type ForumReplyResponse = {
  readonly author: ForumAuthorResponse;
  readonly body: string | null;
  readonly content_state: ForumContentStateResponse;
  readonly created_at: string;
  readonly edited_at: string | null;
  readonly reply_id: string;
  readonly revision: number;
  readonly topic_id: string;
  readonly updated_at: string;
};
