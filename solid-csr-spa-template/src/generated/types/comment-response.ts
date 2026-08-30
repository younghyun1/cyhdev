// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { VoteState } from "./vote-state";

export type CommentResponse = {
  readonly comment_content: string;
  readonly comment_created_at: string;
  readonly comment_id: string;
  readonly comment_updated_at?: string | null;
  readonly parent_comment_id?: string | null;
  readonly post_id: string;
  readonly total_downvotes: number;
  readonly total_upvotes: number;
  readonly user_country_flag?: string | null;
  readonly user_id: string;
  readonly user_name: string;
  readonly user_profile_picture_url: string;
  readonly vote_state: VoteState;
};
