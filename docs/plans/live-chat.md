# Live Chat Plan

## Goal

Add text-only live chat to the site with:

- A compact panel on the main page.
- A dedicated `/live-chat` page.
- Support for logged-in users and guests.
- Public guest IP display and database logging.
- Country flag display for guests and logged-in users using existing in-memory geo-IP and country caches.
- Shared public user info and badge display for blog and chat users.
- WebSocket delivery with typing indicators.
- A 128 MB in-memory chat cache in `ServerState`.
- Full database persistence for all messages.
- Permanent server-side autobans for abnormal message rates.
- No Redis or external realtime infrastructure.

## Backend Scope

Add a live chat domain with database-backed persistence and a bounded in-memory cache.

Proposed Rust modules:

- `rust-be-template/src/domain/live_chat/`
- `rust-be-template/src/dto/requests/live_chat/`
- `rust-be-template/src/dto/responses/live_chat/`
- `rust-be-template/src/handlers/live_chat/`
- `rust-be-template/src/init/load_cache/live_chat.rs`

Add `live_chat_cache` to `ServerState` and initialize it in `ServerStateBuilder`.

On server startup, hydrate the cache from Postgres after the existing cache synchronization steps. The hydration query should load recent messages, enforce the byte budget in Rust, and keep cached messages in chronological order for fast initial WebSocket snapshots.

## Database

Add a migration for `live_chat_messages`.

Proposed columns:

- `live_chat_message_id uuid primary key`
- `room_key varchar not null default 'main'`
- `user_id uuid null references users(user_id)`
- `guest_ip inet null`
- `sender_kind smallint not null`
- `sender_display_name text not null`
- `message_body text not null`
- `message_created_at timestamptz not null`
- `message_edited_at timestamptz null`
- `message_deleted_at timestamptz null`

Proposed indexes:

- `(room_key, message_created_at desc)`
- `(user_id, message_created_at desc)`
- `(guest_ip, message_created_at desc)`

Add a migration for `live_chat_bans`.

Proposed columns:

- `live_chat_ban_id uuid primary key`
- `user_id uuid null references users(user_id)`
- `banned_ip inet null`
- `reason text not null`
- `ban_source varchar not null`
- `banned_at timestamptz not null`
- `expires_at timestamptz null`

`expires_at = null` means permanent. Active/permanent bans should be loaded into `ServerState.live_chat_cache` on startup and indexed in RAM by both user ID and IP.

## Cache Design

Use `scc` containers for the hot cache path. The backend already uses `scc = "3.7.0"`, and the current docs describe it as a high-performance asynchronous/concurrent container crate with async and sync APIs. Relevant containers for chat:

- `scc::HashMap` for write-heavy concurrent key/value state.
- `scc::HashSet` for connection or actor sets if needed.
- `scc::Queue` for lock-free FIFO append and eviction order.
- `scc::TreeIndex` for read-optimized ordered scans by timestamp/key.

Avoid a coarse `RwLock<VecDeque<_>>` as the primary message cache. Chat needs high-throughput append, read-mostly snapshots, typing updates, and bounded eviction; a single lock would make those paths contend unnecessarily.

Proposed shape:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};

pub struct LiveChatCache {
    messages_by_id: scc::HashMap<uuid::Uuid, Arc<CachedChatMessage>>,
    timeline: scc::TreeIndex<ChatTimelineKey, uuid::Uuid>,
    eviction_queue: scc::Queue<ChatEvictionKey>,
    typing_by_actor: scc::HashMap<ChatActorKey, TypingState>,
    connected_clients: scc::HashMap<uuid::Uuid, ChatConnectionState>,
    bans_by_user: scc::HashMap<uuid::Uuid, CachedLiveChatBan>,
    bans_by_ip: scc::HashMap<std::net::IpAddr, CachedLiveChatBan>,
    message_rate_by_key: scc::HashMap<LiveChatRateKey, LiveChatRateState>,
    total_bytes: AtomicUsize,
    message_count: AtomicUsize,
    connected_count: AtomicU64,
    max_bytes: usize,
    broadcast_tx: tokio::sync::broadcast::Sender<LiveChatServerEvent>,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct ChatTimelineKey {
    room_key: ChatRoomKey,
    message_created_at_micros: i64,
    live_chat_message_id: uuid::Uuid,
}

#[derive(Clone, Copy)]
struct ChatEvictionKey {
    live_chat_message_id: uuid::Uuid,
    timeline_key: ChatTimelineKey,
    estimated_bytes: usize,
}
```

`ChatTimelineKey` should be monotonic enough for stable ordering. Include the UUID as the final tie-breaker so messages created within the same timestamp resolution still sort deterministically.

Cache API:

- `sync_live_chat_cache_from_db`
- `append_persisted_chat_message`
- `get_recent_chat_messages`
- `get_chat_messages_before`
- `get_live_chat_cache_stats`
- `subscribe_live_chat_events`
- `set_typing`
- `clear_expired_typing`
- `sync_bans`
- `is_banned`
- `record_message_attempt`

Append path:

1. Build an `Arc<CachedChatMessage>` and `ChatTimelineKey`.
2. Insert into `messages_by_id`.
3. Insert into `timeline`.
4. Push `ChatEvictionKey` into `eviction_queue`.
5. Add the estimated message byte weight to `total_bytes`.
6. Increment `message_count`.
7. Run bounded eviction.

Eviction path:

1. While `total_bytes > max_bytes`, pop from `eviction_queue`.
2. Remove the message from `messages_by_id`.
3. Remove the timeline key from `timeline`.
4. Subtract the stored byte estimate from `total_bytes`.
5. Decrement `message_count`.
6. If a queue entry is stale because another task already removed the message, skip it.

Read path:

- Recent message snapshots should use `timeline` for ordered traversal, then fetch message payloads from `messages_by_id`.
- Older pagination can use cache first if the cursor is inside the cached range, otherwise query Postgres.
- Do not call `Queue::len()` on hot paths; current docs note that `Queue::len()` is `O(N)`. Use atomics for counts and byte usage.

Typing and presence:

- Store typing state in `typing_by_actor`.
- Store per-WebSocket connection metadata in `connected_clients`.
- Use TTL-based cleanup for typing state rather than persistence.
- Presence counters should use atomics and `scc` connection maps, not a global lock.

Notes from current `scc` docs:

- `HashMap` has fine-grained bucket locking and lock-free resizing.
- `HashMap` intentionally does not implement standard `Iterator`; use `iter_async`, `iter_sync`, `retain_*`, or entry traversal.
- `TreeIndex` is read-optimized and supports ordered iteration/range scans, but removed entries are not dropped immediately, so keep large message payloads in `messages_by_id` and store only keys/UUIDs in the index.
- `Queue` is lock-free FIFO and fits eviction order, but it should not be the only message store because recent-page reads would otherwise require full scans.

## WebSocket API

Add public route:

```text
GET /ws/live-chat
```

The route should run after `is_logged_in_middleware`, allowing both logged-in and logged-out connections.

Client events:

```ts
type LiveChatClientEvent =
  | { type: "send_message"; client_message_id: string; body: string }
  | { type: "typing"; is_typing: boolean }
  | { type: "heartbeat"; nonce: string };
```

Server events:

```ts
type LiveChatServerEvent =
  | { type: "hello"; actor: ChatActor; recent_messages: ChatMessage[]; connected_count: number }
  | { type: "message"; message: ChatMessage }
  | { type: "message_ack"; client_message_id: string; message: ChatMessage }
  | { type: "typing"; actor: ChatActor; is_typing: boolean; expires_at: string }
  | { type: "presence"; connected_count: number }
  | { type: "error"; code: string; message: string };
```

Message send path:

1. Validate message length and content using the shared 300-character limit.
2. Resolve actor from session or client IP.
3. Resolve the actor country flag from session country or geo-IP without per-message user-table lookups.
4. Check active bans by IP and user ID.
5. Record the message attempt in a one-second in-memory rate window.
6. If the actor exceeds 10 message events per second, persist a permanent ban, cache it by IP/user, send an error, and close the socket.
7. Persist valid messages to Postgres.
8. Append persisted row to cache.
9. Broadcast to active subscribers.
10. Send `message_ack` to the sender using `client_message_id`.

Typing indicators are ephemeral and should not be persisted.

## REST API

Add cache-backed REST endpoints:

```text
GET /api/live-chat/messages?limit=50
GET /api/live-chat/messages?before_message_id=<uuid>&limit=50
GET /api/live-chat/cache-stats
GET /api/users/:user_name
```

Message pagination is keyset/cursor-based. The first request without `before_message_id` returns the newest cached page in chronological display order. Older pages pass the oldest currently displayed `live_chat_message_id` as `before_message_id`; the backend resolves that cursor and returns older rows from Postgres, also in chronological display order.

The response should include:

- `items`
- `next_before_message_id`
- `has_more`

`cache-stats` should include:

- `max_bytes`
- `used_bytes`
- `message_count`
- `oldest_cached_at`
- `newest_cached_at`
- `active_typing_count`

`/api/users/:user_name` should return public user badge/profile fields only:

- `user_id`
- `user_name`
- `user_created_at`
- `user_country_flag`
- `user_profile_picture_url`

## Guest Identity

For guests:

- Store `guest_ip`.
- Set `sender_kind = guest`.
- Use a display name like `guest@203.0.113.10`.
- Publicly render the IP once in the display name.
- Append the country flag when geo-IP resolution has a matching country cache entry.

Centralize client IP extraction so logging, visitor analytics, and chat use one consistent helper.

## Frontend Scope

Add strict TypeScript DTOs and avoid `any`.

Proposed frontend files:

- `solid-csr-spa-template/src/dtos/requests/live_chat/index.ts`
- `solid-csr-spa-template/src/dtos/responses/live_chat/index.ts`
- `solid-csr-spa-template/src/services/live_chat.ts`
- `solid-csr-spa-template/src/components/LiveChatPanel.tsx`
- `solid-csr-spa-template/src/pages/live_chat.tsx`
- `solid-csr-spa-template/src/pages/user_info.tsx`

Add route:

```ts
{
  path: "/live-chat",
  component: lazy(() => import("./pages/live_chat")),
}
{
  path: "/users/:userName",
  component: lazy(() => import("./pages/user_info")),
}
```

`LiveChatPanel` should support compact and full modes:

```ts
type LiveChatPanelMode = "compact" | "full";
```

Compact mode belongs on the home page. Full mode belongs on `/live-chat`.

Message submission should be optimistic on the client: after a successful WebSocket send, render a local pending message immediately in muted styling, then replace it with the persisted message when `message_ack.client_message_id` returns from the server.

User display should use a common `UserBadge` component in blog and chat. Logged-in chat messages should include the sender profile picture URL and country flag; guests continue to render as public IP labels without a profile link.

## Performance And Safety

Requirements:

- No `unwrap()` or `expect()` in new Rust code.
- Use structured `tracing` logs.
- Persist before broadcast.
- Render all message text as text, not HTML.
- Enforce a 300-character max message length on the client and server.
- Reject oversized WebSocket event frames before JSON parsing.
- Reject empty messages.
- Permanently ban IP/user subjects that exceed 10 message events per second.
- Bound broadcast channels and disconnect slow receivers if needed.
- Use DB queries that avoid N+1 behavior.
- Keep typing events debounced on the client.

## Implementation Sequence

1. Add database migration and Diesel schema updates.
2. Add live chat domain structs and DTOs.
3. Add `LiveChatCache` and startup hydration.
4. Add REST history and cache stats endpoints.
5. Add WebSocket handler with message send, ack, broadcast, typing, and heartbeat.
6. Add Solid DTOs and live chat service.
7. Add reusable `LiveChatPanel`.
8. Mount compact chat on home and full chat on `/live-chat`.
9. Verify with `cargo fmt`, `cargo check`, `cargo clippy`, `npm run typecheck`, `npm run lint`, and `npm run build`.

## References

- `scc` 3.7.0 crate docs: https://docs.rs/scc/latest/scc/
- `scc::Queue` docs: https://docs.rs/scc/latest/scc/queue/struct.Queue.html
- `scc::HashMap` docs: https://docs.rs/scc/latest/scc/hash_map/struct.HashMap.html
- `scc::HashIndex` docs: https://docs.rs/scc/latest/scc/hash_index/struct.HashIndex.html
