// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { PhotographContext } from "./photograph-context";

export type Photograph = {
  readonly photograph_comments: string;
  readonly photograph_context: PhotographContext;
  readonly photograph_created_at: string;
  readonly photograph_id: string;
  readonly photograph_image_type: number;
  readonly photograph_is_on_cloud: boolean;
  readonly photograph_lat: number;
  readonly photograph_link: string;
  readonly photograph_lon: number;
  readonly photograph_shot_at?: string | null;
  readonly photograph_thumbnail_link: string;
  readonly photograph_total_downvotes: number;
  readonly photograph_total_upvotes: number;
  readonly photograph_updated_at: string;
  readonly photograph_view_count: number;
  readonly user_id: string;
};
