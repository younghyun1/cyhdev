# Authentication abuse boundaries

Public authentication endpoints use process-local RAM admission because the backend is intentionally one process. PostgreSQL is not on the throttle path. Restarting clears all windows; running multiple backend processes would give each process an independent budget and is unsupported.

## Admission order

The auth router accepts only POST for mutation and confirmation routes, caps each JSON body at 8 KiB, and applies the source-IP throttle before body buffering or deserialization. OPTIONS and other methods do not consume a budget. After JSON extraction, the handler hashes and admits the normalized email, normalized user name, or opaque token before any account query, Argon2 work, token consume, or email task creation. Login identity budgets count wrong passwords only: the handler checks them before Argon2 work and charges a failure afterward, so knowing an email is not enough to lock its owner out, while the source-IP layer still counts every attempt. A correct password clears the per-source budget. Invalid field syntax is still rejected by the account service.

Authenticated current-password confirmations (`PATCH /api/auth/profile`, `DELETE /api/auth/account`, `POST /api/auth/oidc/link/start`, and `DELETE /api/auth/oidc/link`) charge each attempt to the source IP before Argon2 work and each wrong password to the account. The fifth wrong confirmation in an hour revokes the presenting session and returns 401 with error code 69, so a stolen cookie is not an open password oracle. Email input is capped at 254 bytes, passwords at 128 bytes, and user names at 80 UTF-8 bytes and 20 Unicode scalar values before expensive validation or password work.

The limiter generates a 256-bit process key from operating-system entropy at startup. It stores only SHA-256 digests derived from that key, a low-cardinality endpoint/window discriminator, attempt counts, and monotonic expiry instants. Raw email addresses, user names, IP addresses, and tokens are not retained in limiter state or emitted by rejection logs. IPv4 addresses use the full address; IPv6 addresses share a `/64` source budget.

## Fixed windows and bounds

| Endpoint | Source-IP windows | Identity or token windows |
| --- | --- | --- |
| Login | 10 per minute and 50 per hour | Failures only: email from one source, 5 per 15 minutes; email from all sources, 20 per hour |
| Signup | 10 per hour and 20 per day | Normalized email and user name: 5 per day each |
| Password-reset request | 5 per hour and 20 per day | Normalized email: 1 per 15 minutes and 3 per day |
| Password-reset submission | 10 per 15 minutes | Token digest: 5 per 15 minutes |
| Email verification | 20 per hour | Token digest: 5 per hour |
| Password confirmation | 10 per 15 minutes and 50 per day | Failures only: account, 5 per hour, then session revocation |

Every endpoint and dimension pair has its own table and lock, sized between 4,096 and 8,192 records; together they hold at most 73,728 records, about 11 MiB. One record carries every window of its policy. Admission checks all windows under the table's short mutex critical section and counts the request only when all of them admit it, so a burst rejected by a per-minute window is not also charged to the hourly window. When a table is full, it evicts its oldest record that holds a single attempt: a spray of one-off keys displaces only other one-off keys, whose loss forgives one attempt, while records near a limit are retained so an attacker cannot reset their own budget by flooding. A table full of multi-attempt records rejects a novel key with HTTP 429. A minute job removes expired records. IPv4-mapped IPv6 sources keep their full IPv4 address. Rejections include a rounded-up `Retry-After` value and a structured event containing only endpoint, digest dimension, saturation state, and retry duration.

Argon2 work uses two try-acquire pools: four jobs for login, signup, and password reset, and two for authenticated confirmations, so signed-in accounts cannot keep login at 429. Saturation rejects before another blocking task or memory-hard hash is created. Each job moves its owned permit into the blocking closure, so a cancelled request cannot free the slot while Argon2 is still running. Login performs one Argon2 verification for validly formed credentials whether the email exists or not; missing accounts use a startup-generated dummy hash with the same parameters. New SMTP work similarly requires one of 16 permits before spawning, so authentication traffic cannot create an unbounded task backlog.

## Enumeration behavior

The account-existence route and generated browser contract do not exist. Login returns one 401 response for a missing account or wrong password; only the correct password for an unverified account learns, through a 403, that verification is pending. A valid signup using an existing email returns the same 202 status, 300 ms response floor, and generic body as a newly persisted registration; no persistence-dependent identity or deadline is returned. An unverified duplicate takes over the account with the new submission and receives a fresh verification link through the bounded mail path, while a verified duplicate is a no-op. A user name held by any other account returns one generic 409 whether or not the email exists, because user names are already public profile identifiers.

Password-reset request returns the same success envelope for existing and absent accounts. Both paths perform the common dummy Argon2 work and wait until a 300 ms response floor after admission; absent accounts do not create a token or enqueue email. Issuing a real token atomically deletes all prior reset tokens for that account before inserting one replacement. Reset-token absence, expiry, fabrication, prior use, and concurrent consumption map to one public error. Reset and verification tokens are 256 random bits sent as 43 unpadded base64url characters; PostgreSQL stores only the SHA-256 of that text under a unique index, so a leaked table cannot be replayed. Email links transport tokens in URL fragments; the SPA reads each fragment once, removes it from browser history, and sends the token only in a POST JSON body. Links issued as UUIDs before the digest migration no longer verify.

Sensitive auth responses, including JSON extraction errors and throttle rejections, use `Cache-Control: no-store, max-age=0` and `Referrer-Policy: no-referrer`. Successful auth metadata replaces exact server processing duration with `redacted`.

## Residual timing and availability effects

These controls reduce useful distinctions; they are not a constant-time network protocol. Database cache state, scheduler latency, SMTP permit availability, account-row retrieval, Argon2 implementation variance, and network jitter remain observable. The reset floor is a minimum, not a maximum. Login's common Argon2 work dominates but does not erase the database-row timing difference. Shared NAT addresses and IPv6 `/64` aggregation can cause legitimate clients to share a budget. Capacity saturation deliberately fails closed until expiry cleanup. These tradeoffs keep CPU, memory, database work, and task creation bounded for a single-process personal service.

Forwarded client IPs are disabled unless both `TRUSTED_PROXY_HOPS` and `TRUSTED_PROXY_CIDRS` are configured. The direct socket peer and every intermediate stripped hop must match a trusted CIDR; otherwise the socket peer is authoritative. The configuration is parsed once, hop count is capped at 16, and forwarded-header parsing is capped at 2 KiB.
