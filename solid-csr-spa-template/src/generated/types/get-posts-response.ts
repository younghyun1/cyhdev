// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { PostInfoWithVote } from "./post-info-with-vote";

export type GetPostsResponse = {
  readonly available_pages: number;
  readonly posts: ReadonlyArray<PostInfoWithVote>;
};
