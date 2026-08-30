// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { VoteState } from "./vote-state";

export type PostInfoWithVote = {
  readonly post_created_at: string;
  readonly post_id: string;
  readonly post_is_published: boolean;
  readonly post_published_at?: string | null;
  readonly post_share_count: number;
  readonly post_slug: string;
  readonly post_summary?: string | null;
  readonly post_tags: ReadonlyArray<string>;
  readonly post_title: string;
  readonly post_updated_at: string;
  readonly post_view_count: number;
  readonly total_downvotes: number;
  readonly total_upvotes: number;
  readonly user_country_flag?: string | null;
  readonly user_id: string;
  readonly user_name: string;
  readonly user_profile_picture_url: string;
  readonly vote_state: VoteState;
};
