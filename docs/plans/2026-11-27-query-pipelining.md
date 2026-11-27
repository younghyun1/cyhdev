# Backend query round-trip pass

- Status: complete, 2026-11-27. Intended outcome: fewer database round trips on read paths that issued independent statements one at a time, and one statement where two were keyed on the same row.
- Checkout: branch `perf/query-pipelining`, implementation commit `2367a7ba`, not pushed. No uncommitted paths after this document is committed.
- Completed: see Changes. No migration, schema, HTTP contract, or cache changed.
- Verification: see Verification. No latency was measured.
- Remaining: nothing in scope. The next action is review and merge of the branch; candidates left out on purpose are listed under Not done.

## Problem

Repository reads sent each statement and waited for its rows before sending the next, even when the next statement did not use those rows. A forum topic with a signed-in viewer cost four round trips, a photograph detail five, and a blog post detail up to seven across two pool checkouts. Several account reads also used two statements for facts keyed on one row.

## Changes

- **Joined reads:** statements whose binds do not come from another result are built against a shared `&AsyncPgConnection` and awaited with `tokio::try_join!`. diesel-async 0.10 sends such statements before reading any result. The shared helpers in `src/persistence/public_authors.rs`, `has_current_blog_authority`, and `load_tag_rows` take a shared reference so callers can join them.
- **Comment pages in two stages:** the blog and photograph comment readers are split into the row page, which depends only on the parent identifier, and the enrichment (author batch and viewer votes), which depends on the rows. A detail read joins the row page with its other first-stage statements.
- **Blog post visibility:** the post is read unfiltered with its tags, and blog authority is queried only for a draft. A published post no longer runs the authority statement for a signed-in viewer.
- **One statement instead of two:** the current account and its active picture, the public profile and its picture link, and the forum actor's active state and moderation permission. The picture joins rely on the partial unique index `user_profile_pictures_one_active_per_user`; the public profile keeps its picture conditions in the join so an account without a qualifying picture is still returned.
- **Vote totals:** the four vote paths recount and store totals with one `UPDATE ... RETURNING` under the row lock they already hold, instead of an aggregate followed by an update.
- **Other reads:** reference-data countries with subdivisions, and the four corpus counts in the KCNA archive snapshot, are sent together.
- **Rule recorded:** `docs/architecture/be/feature-boundaries.md` states when statements may be joined and when they must stay sequential.

Round trips per repository call, signed-in viewer:

| Read | Before | After |
| --- | --- | --- |
| Forum topic detail | 4 | 2 |
| Photograph detail | 5 | 2 |
| Photograph comment page | 4 | 2 |
| Photograph page | 3 | 2 |
| Blog post read, published, tags not cached | 3 | 1 |
| Blog post social data | 4 | 2 |
| Blog comment page, published | 4 | 2 |
| Blog listing | 4 | 3 |
| Blog listing presentation data | 2 | 1 |
| Blog posts by identifiers | 2 | 1 |
| Current account, public profile, forum authority | 2 | 1 |
| Vote total store | 2 | 1 |
| Reference countries | 2 | 1 |
| Archive corpus counts | 4 | 1 |

## Decisions that constrain later work

- **Order is not guaranteed.** In diesel-async a statement that must first resolve a custom type awaits that lookup before it takes the statement-cache lock, so a later future can reach the server first on a fresh connection. Only order-independent statements are joined.
- **Locking statements stay sequential.** The `lock_active_user` helpers document a user-then-content lock order, and no `FOR UPDATE` or `FOR SHARE` statement was joined.
- **Write transactions were not pipelined.** One failure aborts the rest of a pipeline inside a transaction, which would blur paths that map specific errors, and the candidates are rare paths (account deletion, startup call cleanup).
- **Missing rows cost more.** A joined read also runs the statements a missing parent would have skipped. Their results are dropped before any return.
- **No new caches.** Listing reads the database by policy in `docs/architecture/be/runtime-cache-policy.md`, and blog and forum authority are database-current by design.
- **No index changes.** Plans on an empty test database are not evidence.

## Verification

All commands ran from the repository root on 2026-11-27 against a throwaway PostgreSQL 18.7 cluster on loopback TCP, removed afterwards.

- Baseline on `2f100342` before any edit: `cargo xtask clippy` passed, `cargo xtask unit` passed, `cargo xtask db-integration` passed 51 cases.
- On `2367a7ba`: `cargo xtask fmt`, `cargo xtask clippy`, and `cargo xtask unit` passed; `cargo xtask db-integration` passed 55 cases, the 51 existing ones and four new ones in `postgres::read_visibility` and `postgres::read_joins`.
- The new cases assert the outcomes a joined read could get wrong: drafts hidden from anonymous and ordinary viewers in detail, comment page, and listing counts, and visible to a manager; missing post, topic, and photograph reported as absent; the viewer's own votes and subscription carried; a deleted photograph owner masked; only the active picture joined; forum authority refused for a missing or unverified account.
- Round trips on the wire were counted with a loopback proxy between the test binary and PostgreSQL. A flight is a client send that follows server data. Three runs per case, same tests on both commits; fixture setup dominates each total.

| Existing case | `2f100342` | `2367a7ba` |
| --- | --- | --- |
| `postgres::forum::forum_enforces_retention_authority_fanout_and_keysets` | 821, 819, 820 | 801, 804, 802 |
| `postgres::comment_threads::` | 1658, 1659, 1658 | 1655, 1654, 1653 |
| `postgres::blog_social::` | 1667, 1669, 1669 | 1666, 1667, 1665 |
| `postgres::photography_invariants::` | 583, 586, 581 | 578, 576, 574 |
| `postgres::account_identity::` | 1009, 1006, 1005 | 1000, 1004, 1000 |

Not run: `cargo xtask migration-rollback` (no migration changed), `cargo xtask openapi` (no route, annotation, or DTO changed), `cargo xtask frontend-check` and browser tests (no frontend change), and every release-profile command. No request latency was measured, and nothing was run against a live service. The per-read table above is derived from the code; only the per-test totals were observed.

## Not done

- Blog post detail still uses two pool checkouts, because the post read runs under the post stripe lock and the social read deliberately runs after it is released.
- Blog listing still reads authority before the count and page, since authority decides their filter, and reads presentation data on a second checkout from the service.
- Page-then-authors reads (topic list, notifications, audit, module list) keep two statements so the deletion-aware author projection stays in one place.
- `messages_before`, `published_search_page`, and `media_cleanup_retry_candidates` feed one result into the next and were left as they are.

## References

- diesel-async 0.10.0 `src/pg/mod.rs`: the Pipelining section of the `AsyncPgConnection` documentation and `with_prepared_statement_after_sql_built`, read from the local registry copy on 2026-11-27. The ordering decision follows from that source; the rest follows from repository code and the tests above.
