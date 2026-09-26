# HTTP and database runtime bounds

The backend serves HTTPS directly with axum-server and a single PostgreSQL pool. This document records the limits that keep slow or abusive clients from holding workers, sockets, or database connections, and how a restart drains. Code is authoritative; constants live next to their owners in `rust-be-template/src/init/` and `src/routers/middleware/`.

## Listener protocol bounds

`init/http_server.rs` installs a Tokio timer on hyper's HTTP/1 and HTTP/2 builders for both the HTTPS listener and the port-80 redirect listener. Without a timer hyper silently disables these limits. HTTP/1 request heads must arrive within 20 seconds; hyper arms the same timer while waiting for the next request on a keep-alive connection, so idle HTTP/1 connections also close after 20 seconds. HTTP/2 connections ping every 30 seconds and close after 20 seconds without an acknowledgement, allow 128 concurrent streams and 64 KiB of request headers, and keep hyper's 20 pending reset streams. TLS handshakes keep axum-server's 10-second deadline, and `TCP_NODELAY` remains as described in [HTTPS transport latency](https-transport.md).

Hyper has no idle timer for an HTTP/2 connection without streams, and a client that completes TLS but sends nothing is not covered by the header timer. Those connections are bounded by admission instead: `init/connection_acceptor.rs` admits each TCP connection before TLS through a limiter shared by both listeners, 4,096 open connections in total and 64 per IPv4 address or IPv6 /64 by default. A refused socket is closed without a handshake. The permit lives inside the stream, so upgraded WebSockets stay counted. `HTTP_MAX_CONNECTIONS` and `HTTP_MAX_CONNECTIONS_PER_IP` override the defaults; invalid values abort startup, and the total should stay below the process descriptor limit.

## Request deadlines

`routers/middleware/request_deadline.rs` wraps the whole router, including static assets and the squaremap files. The deadline covers the handler future and therefore body reading; an expired request returns 503 with `no-store`. Separately, a request body that produces no frame for the idle limit fails, so a stalled upload ends long before a large upload deadline.

| Routes | Deadline | Body idle limit |
| --- | --- | --- |
| Ordinary API, static, map, and WebSocket handshake requests | 30 s | 30 s |
| `/api/admin/*` | 120 s | 30 s |
| `POST` photograph, profile-picture, WASM module, and WASM asset uploads | 15 min | 60 s |
| `POST /api/photographs/batch-upload` (1 GiB limit) | 60 min | 60 s |

Upload deadlines assume roughly 1.4 to 2.4 Mbit/s for a full-size body. Client-supplied `Upgrade` headers never disable these limits. WebSocket handshakes use the ordinary deadline; sessions run after the 101 response and carry their own limits. Response streaming after the head is not timed; HTTP/2 pings and the connection caps bound clients that stop reading.

## WebSocket limits

The host-stats socket (`features/server_status/api/host_stats.rs`) admits 256 sockets and four per client network, limits inbound frames and messages to 1 KiB with a 4 KiB read buffer, writes each 20-byte sample with a five-second deadline, pings every 30 seconds, and closes a socket that sends nothing, including pongs, for 75 seconds. Live-chat socket limits belong to that feature.

## Graceful shutdown

SIGTERM or Ctrl-C stops both listeners from accepting and gives in-flight requests 10 seconds before axum-server closes the remaining connections. Upgraded WebSockets are not part of that drain and end when the process exits. After the listeners stop, `init/shutdown.rs` runs an ordered list of hooks with a shared 10-second budget and at most five seconds each; a hook that fails or overruns is logged and the next one still runs while budget remains. The current hooks close open calls, flush visitor logs, flush photograph view counts, and flush blog view counts, in that order. Add further hooks to `server_shutdown_hooks` with a name and an async closure in the required order.

## Database pool

`init/db_pool.rs` builds the request and job pool with `DB_POOL_MAX_SIZE` connections (default 24, valid 1 to 256), a minimum of four idle connections, and a two-second checkout timeout. Pooled sessions receive `statement_timeout=30s`, `lock_timeout=5s`, and `idle_in_transaction_session_timeout=60s` through the libpq `options` startup parameter, so no extra round trip is needed. The migration runner uses the plain URL and keeps PostgreSQL's unbounded defaults, which covers migration backfills such as the visitor-board projection. A `DB_URL` that already sets `options` is rejected.

No current request or job path needs a longer per-transaction `SET LOCAL statement_timeout`. The longest statements are startup reads (live-chat history is capped at 50,000 rows, the blog cache is one query, and the visitor board now reads a small projection table) and the i18n source synchronization, which upserts at most 1,000 rows per statement in one transaction and skips unchanged values. A future bulk path that needs more time should set `SET LOCAL statement_timeout` inside its own transaction rather than raising the pool default.

bb8 still validates each checkout, but with diesel-async's local broken-connection check (transaction manager state and the tokio-postgres client's closed flag) instead of a `SELECT 1` round trip. A connection whose server went away while idle is discarded at checkout. Database URL components are percent-decoded when parsed from `DB_URL` and percent-encoded when built, so reserved characters in credentials and names reach libpq and tokio-postgres intact; Unix socket directories travel as the `host` query parameter.

## Verification

Unit tests cover the header-read and idle keep-alive timeouts, connection refusal and slot release over real sockets, request classification, deadline and stalled-body behavior, host-stats socket idle and size limits, shutdown hook ordering and budgets, pool settings, and URL encoding. `tests/postgres_runtime_bounds.rs` checks the pooled session settings against PostgreSQL 18 and that the migration connection stays unbounded.
