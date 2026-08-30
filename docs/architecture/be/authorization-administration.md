# Authorization administration

The account feature has four exclusive roles: `younghyun`, `moderator`, `user`, and `guest`. Only an active account whose current PostgreSQL assignment is exactly `younghyun` may read or change authorization state. The RAM session role remains the fast coarse route gate, but it is not the authority for this administration surface. Every read rechecks the current database role. Every change locks all active Younghyun assignments in user-ID order, verifies that the actor is still among them, locks the target rows, applies one change, and inserts its audit event in the same transaction.

Role changes reject no-ops, system actors, deleted accounts, removal of the last active Younghyun, and an actor demoting their own Younghyun assignment. Younghyun is the irreducible owner role, so its seeded permissions cannot be revoked. Stable lock ordering serializes competing owner demotions, so concurrent requests cannot each observe the other owner and leave no administrator. After commit, `AccountService` refreshes every RAM session for the target account from PostgreSQL; a failed refresh revokes those sessions. The service-level session-consistency write lock prevents a login from publishing stale role state during that commit and refresh. Administrative permission checks use current PostgreSQL bindings and are not cached, so binding changes need no cache invalidation.

Permissions use validated lowercase namespace keys such as `authorization.roles.manage`; Rust and PostgreSQL enforce the same 3-to-64-character grammar. The migration seeds a fixed initial catalog and grants it to Younghyun. The administration API changes bindings for existing catalog entries; it does not create arbitrary permission names.

## Audit and privacy

Each committed role or role-permission change writes one UUIDv7 `authorization_audit_events` row containing actor UUID, optional target UUID, role and optional permission snapshots, old and new values, a trimmed 8-to-500-character reason, timestamp, and the request UUID when available. Actor and target UUIDs reference permanent `users` tombstones with `ON DELETE RESTRICT`. User names are never copied into the audit table. Bounded audit reads resolve the current, potentially anonymized display names in one batch query.

The event kind is a native PostgreSQL enum mapped to a Rust enum. Separate triggers reject row update/delete and table truncate. Audit pagination orders by `(created_at DESC, audit_event_id DESC)` and uses the same pair as its cursor and index.

## HTTP surface

- `GET /api/admin/authorization/users`: prefix search over active username or email plus UUIDv7 keyset pagination; `limit` is 1 through 100.
- `GET /api/admin/authorization/roles`: fixed role catalog, capped at 256 rows.
- `GET /api/admin/authorization/permissions`: validated permission catalog, capped at 256 rows.
- `GET /api/admin/authorization/role-permissions`: UUIDv7 keyset pagination; `limit` is 1 through 100.
- `GET /api/admin/authorization/audit`: paired timestamp and UUIDv7 keyset cursor; `limit` is 1 through 100.
- `PATCH /api/admin/authorization/users/{user_id}/role`: requires a matching target confirmation and reason.
- `PATCH /api/admin/authorization/roles/{role_id}/permissions/{permission_id}`: requires matching role and permission confirmations plus a reason.

## Deployment assumption

The backend remains one process with bounded in-RAM sessions. All application role changes must pass through `AccountService` so committed role state is refreshed into those sessions. Direct SQL role edits are operational break-glass work; restart the process afterward to invalidate RAM sessions. The role-administration endpoints still reject stale administrative sessions because they recheck PostgreSQL, but unrelated legacy route gates continue to consume the refreshed RAM role.
