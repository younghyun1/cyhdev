# Live-chat realtime limits

The `/ws/live-chat` socket accepts guests, so every per-connection and per-address resource is bounded before it is spent. Constants live next to their owners in `features/live_chat/api/ws/` and `features/live_chat/service/cache/`; [runtime cache policy](runtime-cache-policy.md) lists the retained maps.

## Guest identity

Guest IP addresses are server side only. They are stored with messages, call participants, and bans, and they key rate limits, connection caps, and bans; no browser-facing format carries them. A guest's public actor key and nickname are an HMAC-SHA256 of the canonical address under `LIVE_CHAT_GUEST_KEY_SECRET`. The secret must be at least 32 bytes; when it is absent or shorter, a random per-process secret is used and identities change on restart. Nicknames of persisted guest messages are re-derived from the same key at load, so older rows never expose the earlier unkeyed IP-derived names. The binary subprotocol is `livechat.bin.v2`; a client offering only v1 falls back to JSON.

## Address groups

IPv4 addresses count individually and IPv4-mapped IPv6 is normalized to IPv4 at the upgrade. IPv6 addresses group by their /64, because one subscriber controls a whole /64. Groups key the message-rate table, the per-address connection cap, and automatic bans. Automatic abnormal-messaging bans cover the group network and expire after 24 hours; manual bans keep their own expiry. Ban lookups also match exact host bans recorded before grouping.

## Connection limits

- **Frames**: tungstenite messages and frames are capped at the 64 KiB protocol limit, so oversized input fails in the codec and closes the connection. Read and write buffers are 16 KiB, and queued outbound bytes are capped at 1 MiB.
- **Malformed input**: four malformed data frames close the connection. Rejections log at debug, with one warning per 10 seconds summarizing the count across all connections.
- **Event budget**: each connection holds a 32-token bucket refilled at 8 tokens per second. Messages cost 2 tokens, typing, heartbeat, ICE, media state, and Ping frames cost 1, leave and answers 2, and a call join 8. An empty bucket drops the event with a `rate_limited` error at most every 2 seconds; 64 consecutive rejections close the connection.
- **Message rate**: more than 10 messages per second from one address group or account persists an automatic ban. When the 16,384-key table is full, a new sender receives a transient `rate_limited` error instead.
- **Connections**: 4,096 in total and 8 per address group; the upgrade answers 429 above the group cap.
- **Liveness**: the server pings every 20 seconds and closes a connection with no inbound frame, Pong included, for 60 seconds. Every socket write times out after 10 seconds, which also ends the reader.

## Broadcasts

Room broadcasts are wrapped once and encoded at most once per wire protocol by the first writer that needs them; every recipient shares those bytes. Deleted-account anonymization runs inside that encode. Typing broadcasts only when the visible typing set changes or a refresh arrives more than 2 seconds after the previous broadcast, coalesced over 250 ms and capped at 16 actors.

## Calls

Call and participant rows without an end time are closed during startup synchronization, since no SFU room outlives the process; `LiveChatService::close_open_calls` is also the hook for graceful shutdown. Slow participants, ICE candidate filtering, and answer handling are described in the [SFU design](rtc-sfu.md).

## Browser history window

The chat panel keeps at most 300 entries and trims the end farther from the reader: live messages at the bottom drop the oldest, older pages drop the newest, and a full window is never trimmed under a reader scrolled into history. A window that no longer reaches the newest message buffers up to 50 live messages and reloads the latest page when the reader returns to the bottom or sends. The logic and its tests live in `solid-csr-spa-template/src/services/live_chat_history.ts`.

## Verification

Unit tests cover the codec (no address bytes on the wire), keyed identity against RFC 4231 vectors, address grouping, rate-table saturation and sweeps, prefix bans, connection caps, typing coalescing, the event budget, shared broadcast encoding, a socket-level oversized-frame test, and the ICE policy. `tests/postgres_live_chat_runtime.rs` covers prefix-ban expiry, room-scoped history, tied row-comparison cursors, and call reconciliation against PostgreSQL 18.
