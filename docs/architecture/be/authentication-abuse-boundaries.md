# Authentication abuse boundaries

Public authentication endpoints use process-local RAM admission because the backend is intentionally one process. PostgreSQL is not on the throttle path. Restarting clears all windows; running multiple backend processes would give each process an independent budget and is unsupported.

## Admission order

The auth router accepts only POST for mutation and confirmation routes, caps each JSON body at 8 KiB, and applies the source-IP throttle before body buffering or deserialization. OPTIONS and other methods do not consume a budget. After JSON extraction, the handler hashes and admits the normalized email, normalized user name, or opaque token before any account query, Argon2 work, token consume, or email task creation. Invalid field syntax is still rejected by the account service. Email input is capped at 254 bytes, passwords at 128 bytes, and user names at 80 UTF-8 bytes and 20 Unicode scalar values before expensive validation or password work.

The limiter generates a 256-bit process key from operating-system entropy at startup. It stores only SHA-256 digests derived from that key, a low-cardinality endpoint/window discriminator, attempt counts, and monotonic expiry instants. Raw email addresses, user names, IP addresses, and tokens are not retained in limiter state or emitted by rejection logs. IPv4 addresses use the full address; IPv6 addresses share a `/64` source budget.

## Fixed windows and bounds

| Endpoint | Source-IP windows | Identity or token windows |
| --- | --- | --- |
| Login | 10 per minute and 50 per hour | Normalized email: 5 per 15 minutes |
| Signup | 3 per hour and 10 per day | Normalized email and user name: 2 per day each |
| Password-reset request | 5 per hour and 20 per day | Normalized email: 1 per 15 minutes and 3 per day |
| Password-reset submission | 10 per 15 minutes | Token digest: 5 per 15 minutes |
| Email verification | 20 per hour | Token digest: 5 per hour |

The source map holds at most 16,384 window records; the identity/token map holds at most 32,768. One identity may occupy multiple records when a policy has short and long windows. Admission checks all records under one short mutex critical section, so capacity and counters cannot race past their limits. A novel key is rejected with HTTP 429 when its map is full. A minute job removes expired records; an existing expired key can also replace its own records in place. Rejections include a rounded-up `Retry-After` value and a structured event containing only endpoint, digest dimension, saturation state, and retry duration.

Argon2 work has a separate four-job try-acquire semaphore shared by login, signup, password reset, profile update, and account deletion. Saturation rejects before another blocking task or memory-hard hash is created. Login performs one Argon2 verification for validly formed credentials whether the email exists or not; missing accounts use a startup-generated dummy hash with the same parameters. New SMTP work similarly requires one of 16 permits before spawning, so authentication traffic cannot create an unbounded task backlog.

## Enumeration behavior

The account-existence route and generated browser contract do not exist. Login returns one 401 response for a missing account or wrong password. A valid signup using an existing email returns the same 202 status, 300 ms response floor, and generic body as a newly persisted registration; no persistence-dependent identity or deadline is returned. An active unverified duplicate receives a replacement one-time verification capability through the bounded mail path, while verified and absent identities are no-ops. A duplicate public user name may return one generic 409 because user names are already public profile identifiers.

Password-reset request returns the same success envelope for existing and absent accounts. Both paths perform the common dummy Argon2 work and wait until a 300 ms response floor after admission; absent accounts do not create a token or enqueue email. Issuing a real token atomically deletes all prior reset tokens for that account before inserting one replacement. Reset-token absence, expiry, fabrication, prior use, and concurrent consumption map to one public error. Reset and verification email links transport tokens in URL fragments; the SPA reads each fragment once, removes it from browser history, and sends the token only in a POST JSON body.

Sensitive auth responses, including JSON extraction errors and throttle rejections, use `Cache-Control: no-store, max-age=0` and `Referrer-Policy: no-referrer`. Successful auth metadata replaces exact server processing duration with `redacted`.

## Residual timing and availability effects

These controls reduce useful distinctions; they are not a constant-time network protocol. Database cache state, scheduler latency, SMTP permit availability, account-row retrieval, Argon2 implementation variance, and network jitter remain observable. The reset floor is a minimum, not a maximum. Login's common Argon2 work dominates but does not erase the database-row timing difference. Shared NAT addresses and IPv6 `/64` aggregation can cause legitimate clients to share a budget. Capacity saturation deliberately fails closed until expiry cleanup. These tradeoffs keep CPU, memory, database work, and task creation bounded for a single-process personal service.

Forwarded client IPs are disabled unless both `TRUSTED_PROXY_HOPS` and `TRUSTED_PROXY_CIDRS` are configured. The direct socket peer and every intermediate stripped hop must match a trusted CIDR; otherwise the socket peer is authoritative. The configuration is parsed once, hop count is capped at 16, and forwarded-header parsing is capped at 2 KiB.
