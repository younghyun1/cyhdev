// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { PostInfoWithVote } from "./post-info-with-vote";

export type SearchPostsResponse = {
  readonly available_pages: number;
  readonly page: number;
  readonly posts: ReadonlyArray<PostInfoWithVote>;
  readonly query: string;
  readonly search_type: string;
};
