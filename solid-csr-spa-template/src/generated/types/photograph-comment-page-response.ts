// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { CommentCursorResponse } from "./comment-cursor-response";
import type { PhotographCommentResponse } from "./photograph-comment-response";

export type PhotographCommentPageResponse = {
  readonly comments: ReadonlyArray<PhotographCommentResponse>;
  readonly next_cursor: CommentCursorResponse | null;
};
