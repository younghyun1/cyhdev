// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type LiveChatCacheStatsResponse = {
  readonly active_typing_count: number;
  readonly connected_count: number;
  readonly max_bytes: number;
  readonly message_count: number;
  readonly newest_cached_at?: string | null;
  readonly oldest_cached_at?: string | null;
  readonly used_bytes: number;
};
