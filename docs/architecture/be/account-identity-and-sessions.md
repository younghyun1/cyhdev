# Account identity and session invariants

The account slice treats `users.user_email` and `users.user_name` as exact, case-sensitive identities because current registration, login, and public lookup behavior does not normalize either value. PostgreSQL enforces those identities through the named `users_user_email_unique` and `users_user_name_unique` constraints. Registration classifies conflicts by constraint name without a race-prone existence query. The service retains the detailed constraint result for coordination; the public signup endpoint suppresses duplicate-email detail behind the same accepted response as a new registration and maps a duplicate public user name to one generic conflict.

The identity migration takes an exclusive lock and fails closed when pre-existing duplicates exist. It never deletes, renames, or merges an account. An operator must choose the canonical account, repair references and identity values explicitly, and rerun the migration. Its down migration restores the prior non-unique lookup indexes.

All account, email-verification, password, and role mutations are coordinated by `AccountService`. Repositories commit database changes; the service then updates the process-local session store. Password changes and account removal revoke sessions. Verification and role changes refresh sessions from committed data; if that authoritative read fails, the service revokes every affected session before returning the committed mutation result. A bounded read/write coordination gate lets logins proceed concurrently while excluding session-affecting mutations through commit and refresh or revocation. This prevents an in-flight old-password login from recreating a session after a password reset, and prevents concurrent role or verification changes from publishing stale session state.

Sessions are intentionally process-local because this backend is deployed as one process. PostgreSQL has no sessions table and authenticated requests do not query the database. Restarting the process invalidates every session by design; running multiple backend processes would create independent, incoherent login state and is unsupported.

## Session credential and lookup

Each successful login obtains 32 bytes from the operating-system CSPRNG. The browser receives those bytes as a 43-character unpadded base64url value in the `__Host-cyhdev-session` cookie. The cookie is host-only because it has no `Domain` attribute; it also has `Secure`, `HttpOnly`, `SameSite=Strict`, `Path=/`, and a one-hour `Max-Age`. JavaScript cannot read it, but the browser sends it on credentialed same-origin requests.

The session store never retains the bearer token. It computes `SHA-256(secret)` and uses the full 32-byte digest as the key in a concurrent `scc::HashMap`. Middleware rejects any token that is not exactly 43 bytes of canonical base64url, decodes into a fixed stack buffer, hashes it, and performs one expected O(1) map lookup. Invalid input cannot cause an input-sized allocation. A successful lookup clones only bounded session authority; the user name uses `Arc<str>` so the common clone does not copy its allocation.

Sessions expire after one hour. Lookup removes an expired entry immediately. The maintenance purge and capacity admission path also remove expired entries. The store reserves admission with an atomic counter before insertion, so concurrent logins cannot exceed the fixed 16,384-session limit. When full, admission purges expired entries once and then returns an explicit HTTP 503 error rather than growing memory or evicting an active user.

Login rotates only the valid session token already presented by that browser. Other browsers and devices retain independent sessions. Password and identity changes revoke every session for the user; verification, profile authority, and role changes refresh every unexpired session from committed account data. Logout removes only the presented session. The raw token and its derived key are never logged.

## Security boundary

The cookie replaces a shared frontend API key, not browser trust. A copied cookie remains a bearer credential until its one-hour expiry, explicit logout, user-wide revocation, or process restart. `HttpOnly` prevents direct cookie reads through page JavaScript, but an active same-origin script injection can still issue requests as the user. Exact trusted-origin checks on unsafe requests and WebSocket handshakes remain required alongside output encoding and the application content-security policy.
