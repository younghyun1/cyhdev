// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type Post = {
  readonly post_content: string;
  readonly post_created_at: string;
  readonly post_id: string;
  readonly post_is_published: boolean;
  readonly post_metadata: unknown;
  readonly post_published_at?: string | null;
  readonly post_share_count: number;
  readonly post_slug: string;
  readonly post_summary?: string | null;
  readonly post_title: string;
  readonly post_updated_at: string;
  readonly post_view_count: number;
  readonly total_downvotes: number;
  readonly total_upvotes: number;
  readonly user_id: string;
};
