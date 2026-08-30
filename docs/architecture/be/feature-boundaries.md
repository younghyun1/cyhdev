# Backend feature boundaries

Backend code is organized as feature-local vertical slices. The account and authentication slice at `rust-be-template/src/features/accounts` is the reference layout; other features should follow its boundaries without importing account-specific implementation details.

## Layout and dependency direction

Each feature uses this shape:

```text
features/<feature>/
├── api/
├── domain/
├── repository/
├── service/
└── mod.rs
```

Every directory contains a `mod.rs` with module declarations only. Optional feature-wide files, such as `error.rs`, may sit beside these directories when their ownership is unambiguous.

Dependencies point inward through the use case:

```text
composition root -> api -> service -> repository interface
                           |                 |
                           +----> domain <---+
```

The composition root constructs infrastructure and service dependencies. `api` may depend on `service` and protocol DTOs. `service` may depend on domain types and narrow repository or infrastructure interfaces. A repository implementation may depend on Diesel, the generated schema, the connection pool, domain types, and the deliberately small adapters under `crate::persistence`. `domain` must not depend on Axum, Diesel, the database schema, application state, caches, or session storage. Reverse imports and cross-feature access to another feature's repository are not allowed.

`crate::persistence` contains only cross-feature relational primitives that must execute inside a caller-owned transaction: active-account row locks, bounded public-author projections, and durable media-cleanup registration. These adapters prevent feature repositories from importing another feature's repository while preserving deletion ordering, privacy projection, and metadata-to-object-store outbox atomicity. They do not own use cases, pools, caches, network calls, or transaction scope; adding an adapter requires a concrete same-transaction invariant that a service port cannot preserve.

## Layer ownership

### API

The `api` layer owns route assembly, Axum extractors, request and response DTOs, protocol validation, authentication-context extraction, HTTP status selection, cookies, headers, and mapping feature errors into the stable API error contract. A handler should parse and authorize the request, call one service use case, and map the result.

Handlers do not acquire database connections, compose Diesel queries, begin transactions, hash passwords, send mail, or update caches and sessions directly. They must not import `diesel`, `schema`, row structs, pool types, or general application-state business methods.

### Service

The `service` layer owns use cases, business authorization, invariants spanning more than one domain value, transaction scope, and coordination among repositories and infrastructure. Examples include authenticating credentials, registering an account and its verification token atomically, changing a password, assigning a role, refreshing affected sessions, and invalidating account caches after a committed mutation.

A service decides which database operations must be atomic; the repository or a feature-local unit-of-work interface implements that transaction with Diesel. Services never receive a raw connection and never import Diesel or the database schema. Slow CPU work such as password hashing and blocking infrastructure calls must stay off Tokio worker threads.

Service dependencies are explicit constructor parameters or fields. Clocks, token generation, password hashing, mail delivery, caches, and session stores are dependencies when a use case needs them; they are not reached through globals or an all-purpose `ServerState`.

### Repository

The `repository` layer owns pool checkout, Diesel query construction, persistence row types, row-to-domain mapping, database transactions, database error normalization, and query performance. Its public methods accept domain values and return domain values or repository errors, not Axum DTOs, Diesel rows, pooled connections, or schema types.

Queries use Diesel's query builder and the generated schema. Aggregate reads should use joins or bounded batched queries; per-row lookups that create N+1 behavior are not acceptable. Transactions should be short and contain database work only. Network calls, password work, cache locks, session-map iteration, and email delivery happen outside the transaction. An external side effect that must be reliable with a commit requires a durable outbox or another explicit recovery mechanism; it must not be hidden inside a database transaction callback.

### Domain

The `domain` layer owns persistence-independent values, enums, commands, results, and state rules. Types should make invalid states hard to construct and should use rich types or newtypes where a primitive would admit invalid values. A domain value is not an HTTP representation or a database record merely because the current fields happen to match.

Domain code may depend on small general-purpose crates for values such as time and UUIDs. It must not know about routes, cookies, serialization policy, SQL tables, pool errors, cache implementations, or application state.

## Explicit wiring

Infrastructure is assembled once at the application composition root. It builds the account repository from the pool, builds bounded cache and session-store adapters, constructs the account service from those dependencies, and exposes a clonable service handle to the account API. Handlers extract that handle and invoke a use case; they do not use `ServerState::get_conn` or business methods attached to `ServerState`.

Application state may retain process-wide infrastructure required for composition and lifecycle management, but feature behavior belongs on feature services. Cross-feature work calls a narrow service or port owned by the providing feature. It never reaches through state into another feature's tables or internal cache.

## Transactions, queries, caches, and sessions

- **Transactions:** The service defines the atomic business boundary. A repository method or unit of work executes it and returns only after commit or rollback. Do not hold a transaction open across external I/O or expensive computation.
- **Queries:** Repositories own query shape, projection, batching, ordering, locking, and conversion from constraint failures into stable repository errors. Services express intent rather than assembling filters.
- **Caches:** Services decide when a use case may read, populate, refresh, or invalidate a cache. Cache adapters own storage mechanics, size bounds, expiry, and metrics. A cache is never the record of authority, and runtime-growing unbounded caches are forbidden.
- **Sessions:** Account services own session creation, rotation, refresh, and revocation policy. A session-store dependency owns storage and lookup mechanics. Account, email, password, and role mutations update or revoke all affected sessions only after the database commit succeeds.

When post-commit cache or session work fails, the service records a structured failure and returns an outcome that distinguishes the committed mutation from the failed maintenance work. Repair and client retry paths must be idempotent; the service must not report that a committed database mutation rolled back. Prefer invalidation and authoritative re-read over writing speculative state before commit.

## Errors and observability

Repositories translate pool, Diesel, and constraint failures into repository errors with enough structure for the service to distinguish conflicts, absence, retryable failures, and internal faults. Services translate those into feature errors without HTTP status codes. The API performs the final feature-error-to-HTTP mapping.

Log each failure once at the layer that has enough context to act on it. Use structured fields and avoid logging passwords, tokens, cookie values, password hashes, or complete account records. Do not erase a source error before the layer responsible for diagnosis has recorded or preserved it.

## Module and file limits

- Keep every handwritten Rust source file below 300 lines. Split by use case, aggregate, or query family before it crosses that limit. Deterministic generated contract data such as Diesel's `schema.rs` and the fixed UI-text key catalog are exempt; their generated header and drift check must make that status explicit.
- Keep `mod.rs` files to module declarations. Put route assembly, constructors, shared types, and implementations in named files.
- Default to private or `pub(crate)` visibility. Export only the service surface, domain values, and ports required by callers.
- Name files for behavior or data ownership, such as `api/login.rs`, `service/authenticate.rs`, `repository/accounts.rs`, and `domain/account.rs`.
- Keep API DTOs, persistence records, and domain values distinct. Conversion at a boundary should be explicit and local.
- Unit-test domain rules and service orchestration without Axum or PostgreSQL. Test repository queries against PostgreSQL, including constraints, transaction rollback, batching, and concurrent updates. Test API mapping at the router boundary.

## Incremental migration

Migrate one coherent vertical slice at a time. Accounts and authentication are first because later account mutations, bounded process-local sessions, OAuth/OIDC, and role administration depend on this boundary.

1. Capture the current route, request, response, status, cookie, authorization, and database behavior. A boundary refactor does not change the public contract or schema unless that change is separately scoped.
2. Introduce persistence-independent domain inputs and results for one use case.
3. Move all Diesel records and queries for that use case into the feature repository. Combine related writes into a repository transaction chosen by the service's atomicity requirement.
4. Move validation, authorization, cryptography, transaction coordination, cache invalidation, and session policy into the service.
5. Wire explicit dependencies at the composition root and reduce the handler to extraction, one service call, and response mapping.
6. Move callers to the new service before removing legacy methods. A temporary adapter is acceptable when it delegates in one direction; parallel implementations, dual writes, and circular dependencies are not.
7. Remove obsolete handlers, domain aliases, `ServerState` business methods, and exports only after repository-wide search proves there are no consumers.

Do not mix migrations for overlapping features in one pass. A later feature may proceed once the account reference slice is stable, but each active feature keeps ownership of its own `api`, `service`, `repository`, and `domain` tree. Cross-cutting composition changes are serialized.

The account reference slice is complete when its handlers contain no Diesel or connection acquisition, its services contain no Axum or schema access, repositories expose no HTTP or cache/session behavior, account business methods no longer live on `ServerState`, public API behavior remains stable, and the staged Clippy command passes. Remaining features then migrate to the same boundaries incrementally.
