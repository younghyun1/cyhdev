// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { CommentCursorResponse } from "./comment-cursor-response";
import type { CommentResponse } from "./comment-response";

export type CommentPageResponse = {
  readonly comments: ReadonlyArray<CommentResponse>;
  readonly next_cursor: CommentCursorResponse | null;
};
