// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { VoteState } from "./vote-state";

export type PhotographCommentResponse = {
  readonly parent_photograph_comment_id?: string | null;
  readonly photograph_comment_content: string;
  readonly photograph_comment_created_at: string;
  readonly photograph_comment_id: string;
  readonly photograph_comment_total_downvotes: number;
  readonly photograph_comment_total_upvotes: number;
  readonly photograph_comment_updated_at?: string | null;
  readonly photograph_id: string;
  readonly user_country_flag?: string | null;
  readonly user_id: string;
  readonly user_name: string;
  readonly user_profile_picture_url: string;
  readonly vote_state: VoteState;
};
