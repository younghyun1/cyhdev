# Forum architecture

The forum is an independent feature slice under `features/forum`; it does not reuse blog comments. Topics have one opening plain-text body and flat replies. Titles contain 3-160 characters and at most 512 bytes. Topic and reply bodies contain 1-20,000 characters and at most 65,536 bytes. The route body limit is 128 KiB so JSON escaping cannot reject a domain-valid body. No Markdown or HTML rendering occurs.

## State and attribution

Topics and replies use native `visible`, `hidden`, and `deleted` content states. Topics separately use native `open` and `locked` access state and a pinned flag. Author edits and deletes carry an expected positive revision; moderation also carries an expected revision. Each successful mutation increments the row revision, so concurrent clients receive a stable conflict instead of overwriting newer state. Author deletion replaces title/body or reply body with `[deleted]`, removes it from visible full-text search, and keeps a stable row and attribution. Moderator hiding retains the original text for an explicit restore but public projections return nullable masked bodies. When a topic is hidden or deleted, its reply bodies are also masked. User foreign keys use `ON DELETE RESTRICT` because account lifecycle retains permanent tombstones; the shared batched `PublicAuthor` projection masks deleted account identity without N+1 queries.

`forum_topic_reply_count` is the total number of retained reply rows, including hidden and deleted tombstones. It increments once in the reply transaction and never falls. `last_activity_at` and the count update in the same transaction as reply creation. Reply transactions take the topic row lock and retain the maximum stored/request timestamp, so an older request that acquires the lock late cannot move activity or update time backward.

## Reads and search

Normal topic lists use the keyset `(is_pinned DESC, last_activity_at DESC, topic_id DESC)` with a 1-100 page bound. Pinned topics are the first partition; each partition is recency-first. Search accepts at most 128 characters, 512 bytes, and 16 whitespace-separated terms. A stored `simple`-configuration `tsvector` over title and opening body has a partial GIN index for visible topics; matching rows ignore pin priority and use the strictly recency-first `(last_activity_at DESC, topic_id DESC)` index and cursor. Topic detail uses the ascending `(reply_created_at, reply_id)` keyset. Normal non-search lists retain hidden/deleted tombstones; search never matches them.

## Writes and authority

Every forum write obtains an account-authority lease. The lease holds the account service's process-local consistency read guard while PostgreSQL confirms the account remains active and locally email-verified, then remains held through the forum transaction. Account deletion, role assignment, and permission changes take the write guard. Moderation additionally queries current PostgreSQL `forum.moderate` permission on every decision. Migration `060000_forum` seeds that collision-safe permission and grants it to Younghyun; role administration can grant it elsewhere later.

A fixed 16,384-actor RAM limiter admits at most three topic creates and 30 reply creates per account in each ten-minute window. Payload validation occurs before charging. Saturation and exhausted windows return 429 with `Retry-After`; expired actors are purged only when capacity admission needs space. No auxiliary queue or runtime-growing cache exists.

## Subscriptions and notifications

Topic creators and repliers auto-subscribe while capacity remains. Explicit subscribe/unsubscribe endpoints are also available. A locked topic row serializes subscription admission and enforces 4,096 subscribers per topic. Saturation rejects an explicit subscription but does not block a valid reply; the replier simply remains unsubscribed. Reply creation uses one `INSERT ... SELECT` from that bounded subscription set to create exactly one notification per recipient/reply while excluding the actor. The notification insert, reply insert, topic counter/activity update, and auto-subscription share one transaction; no subscriber vector or N+1 insert loop is used.

Notifications expire exactly 90 days after creation. Inbox reads ignore expired rows even if maintenance is delayed. An hourly job locks and deletes at most 512 expired rows with `SKIP LOCKED`, then reports whether more remain. Topic deletion removes its now-unusable subscription rows in the same transaction while retaining historical notifications. Soft account deletion atomically removes that account's subscriptions and recipient inbox while retaining authored topics, replies, notification actor attribution, and moderation audit attribution.

## Moderation and rollback

Topic actions are hide, restore, lock, unlock, pin, and unpin. Reply actions are hide and restore. Every action requires an 8-500 character, 2,000-byte reason and appends a native-enum audit row containing actor, exact target, optional request UUID, and timestamp. Database triggers reject update, delete, and truncate of audit rows.

The down migration refuses to run once any forum content, subscription, notification, or moderation audit exists. It also refuses if the seeded permission or Younghyun binding changed, or if authorization audit references that permission. Only an untouched seed may be removed.

## HTTP surface

Public reads are `GET /api/forum/capabilities`, `GET /api/forum/topics`, and `GET /api/forum/topics/{topic_id}`. Verified sessions can create, edit, and delete topics/replies; manage `/subscription`; list `/notifications`; and mark `/{notification_id}/read`. Permission holders use topic/reply `/moderation` and `GET /api/forum/moderation/audit`. All state-changing routes inherit the exact trusted-origin policy and secure RAM-session middleware.
