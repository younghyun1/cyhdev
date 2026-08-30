// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { Photograph } from "./photograph";
import type { PhotographCommentResponse } from "./photograph-comment-response";
import type { UserBadgeInfo } from "./user-badge-info";
import type { VoteState } from "./vote-state";

export type ReadPhotographResponse = {
  readonly comments: ReadonlyArray<PhotographCommentResponse>;
  readonly photograph: Photograph;
  readonly user_badge_info: UserBadgeInfo;
  readonly vote_state: VoteState;
};
