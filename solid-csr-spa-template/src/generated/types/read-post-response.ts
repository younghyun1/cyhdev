// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { CommentResponse } from "./comment-response";
import type { Post } from "./post";
import type { UserBadgeInfo } from "./user-badge-info";
import type { VoteState } from "./vote-state";

export type ReadPostResponse = {
  readonly comments: ReadonlyArray<CommentResponse>;
  readonly post: Post;
  readonly post_tags: ReadonlyArray<string>;
  readonly user_badge_info: UserBadgeInfo;
  readonly vote_state: VoteState;
};
