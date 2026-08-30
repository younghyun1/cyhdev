# Photography: Batch Upload + Processing Tracker, then Views/Votes/Comments

## Context

The photography gallery currently uploads one file at a time. `POST /api/photographs/upload` (`src/handlers/photography/upload_photograph.rs`) reads the whole file into memory, encodes EXIF + main + thumbnail inline (`tokio::join!`), does two sequential S3 PUTs, inserts a row, and only then returns the `Photograph`. The client (`solid-csr-spa-template/src/pages/photographs.tsx`) is single-file and blocks on that full round trip. The handler itself carries a TODO (`upload_photograph.rs:52`): "STREAM to file, don't keep the whole damn thing around."

Two goals, sequenced:

1. **Batch upload + processing tracker (first).** Upload many photos at once. The API replies immediately with a batch-session UUID; encoding runs in parallel in the background; an in-memory, per-user, private status tracker drives a "Processing" popup next to the Upload button.
2. **Social features (second).** Expose photograph view counts and make photographs votable and commentable, mirroring the existing blog policy but with separate tables. Same visual style.

Both are additive. The legacy `upload_photograph` endpoint stays (still used for `PhotographContext::Post` images); the gallery's multi-upload uses the new batch endpoint.

## Decisions (from the user)

- **Per-file metadata = per-file mini-forms.** Each selected file gets its own comment + location in a list; no shared/EXIF fallback. Submit is blocked until every file has comment + lat/lon (Photography context).
- **View counts = naive +1, mirror blog.** Increment on detail open, no dedup.
- **Build order = batch upload first, then social.**
- **Privacy = return 404 (not 403)** for a batch that is absent or not owned by the caller. This is load-bearing: the frontend's `apiFetch`/`get` 401/403 handler clears auth and hard-redirects to `/login`; a 403 from the status poller would nuke the user's session mid-upload.

## Conventions to follow (verified)

- `uuid` already has the `v7` feature and `num_cpus` is a dependency; `tokio` is `features=["full"]`. Use `Uuid::now_v7()` for new IDs; legacy code uses v4.
- In-memory cache house pattern: `scc::HashMap`/`TreeIndex` + `Atomic*` counters, async `insert_async`/`update_async`/`retain_async`/`scan_async` (`src/domain/live_chat/cache.rs`). `ServerState` holds caches as fields (`src/init/state/server_state.rs`), initialized in `src/init/state/builder.rs`, with `impl ServerState` split into submodules under `src/init/state/server_state/`.
- Handlers: `State(state): State<Arc<ServerState>>` first, then `Extension(user_id): Extension<Uuid>` / `Path` / `Json`; return `HandlerResponse<impl IntoResponse>`; wrap success in `http_resp(data, (), start)` with `start = tokio_now()`; errors via `code_err(CodeError::X, e)`. Acquire DB conn late and `drop(conn)` right after the query.
- No `unwrap`/`expect`; explicit `match` on `Option`/`Result`; structured `tracing` with fields; files <300 LOC (split into folders whose `mod.rs` holds only declarations); rustdoc on modules/functions.
- Routes registered in `src/routers/main_router.rs` across public / protected (auth) / superuser tiers. Periodic jobs use the `supervise(...)` + `schedule_task_every_minute_at(...)` pattern in `src/jobs/job_funcs/init_scheduler.rs`.
- Frontend: plain exported `createSignal`/`createStore` modules for global state (`src/state/*.ts`), NOT SolidJS Context. Styling via `src/styles/pageStyles.ts` tokens + the inline `<style>` block in `photographs.tsx` (no real `.css`). i18n is strictly typed: every new key must be added to `src/i18n/keys.ts` AND both `src/i18n/defaults/en-us.ts` and `ko-kr.ts` or `tsc` fails.
- Column naming: new columns and primary keys use the table-prefixed convention (`photograph_view_count`, `photograph_vote_id`, ...), even though blog's legacy columns are unprefixed (`vote_id`, `total_upvotes`).

---

## Phase 1 — Batch upload + processing tracker

### Backend

**Tracker types** — new `src/domain/photography/batch.rs` (split into `batch/{mod,status,session}.rs` if >300 LOC; add `pub mod batch;` to `src/domain/photography/mod.rs`):

- `ProcessingStatus` — serde-tagged enum (`#[serde(tag = "status", rename_all = "snake_case")]`, mirror `src/domain/live_chat/cache/event.rs`): `Queued`, `Encoding`, `Uploading`, `Persisting`, `Completed { photograph_id, photograph_link, thumbnail_link }`, `Failed { reason }`. `is_terminal()` helper.
- `BatchItem { item_id: Uuid (now_v7), original_file_name: Option<String>, original_size_bytes: u64, status: ProcessingStatus, created_at, updated_at }`.
- `BatchSession { batch_id: Uuid, owner: Uuid, created_at, total: usize, items: scc::HashMap<Uuid, BatchItem>, completed: AtomicUsize, failed: AtomicUsize, last_activity: AtomicI64 }`. Methods `set_status`/`complete_item`/`fail_item` update the item via `update_async` and bump the atomic only on a real non-terminal→terminal transition (idempotent: guard on `!was_terminal`). `pending = total - completed - failed`; `done = completed + failed >= total`.

**ServerState wiring** — field `photograph_batches: scc::HashMap<Uuid, Arc<BatchSession>>` in `src/init/state/server_state.rs` (+ `mod photography_batches;`), init in `builder.rs`. New `src/init/state/server_state/photography_batches.rs` with `register_batch`, `get_owned_batch(batch_id, requester) -> Option<Arc<BatchSession>>` (returns `None` for absent OR not-owned — single 404 path, no enumeration), `list_owned_batches(requester)`, `prune_terminal_batches(now)`.

**Endpoints** (all in `superuser_router`, mirroring the existing upload route's tier):

- `POST /api/photographs/batch-upload` — `src/handlers/photography/batch_upload.rs`. Multipart: ordered `files` parts + a single `meta` JSON sidecar field (array aligned to file order: `[{comment, lat, lon}, ...]`) + `context`. Validate `meta.len() == files.len()` and required fields per `PhotographContext`. **Stream each file part to a temp file** (`std::env::temp_dir()/cyhdev-batch/{batch_id}/{item_id}.orig` via `tokio::fs`) — closes the line-52 TODO and bounds memory. Reject empty (`BATCH_EMPTY`) or over-count (`BATCH_TOO_MANY_FILES`, e.g. `MAX_FILES_PER_BATCH = 50`). Compute `total` from successfully-staged files, register the session, spawn the pipeline, return **202** `{ batch_id, total, items: [{item_id, file_name}] }` immediately. Apply a route-scoped `DefaultBodyLimit::max(BATCH_REQUEST_SIZE)` (≈1 GB) on a nested router merged into `superuser_router` so the global 150 MB limit isn't widened elsewhere.
- `GET /api/photographs/batch/{batch_id}` — `batch_status.rs`. `Extension<Uuid>` + `get_owned_batch`; 404 `BATCH_NOT_FOUND` when absent/not-owned. Returns `{ batch_id, created_at, total, completed, failed, pending, done, items: [...] }` (aggregate from atomics, O(1); items serialized O(N)).
- `GET /api/photographs/batches` — `batch_list.rs`. Caller's active batches via `list_owned_batches`.

DTOs in new `src/dto/responses/photography/batch_status_response.rs` (`#[derive(Serialize, ToSchema)]`). New error codes 47/48/49 in `src/errors/code_error.rs` (`BATCH_NOT_FOUND` 404, `BATCH_TOO_MANY_FILES` 400, `BATCH_EMPTY` 400); no 403 code (collapsed into 404).

**Pipeline** — new `src/util/image/batch_pipeline.rs` (`spawn_batch` + `process_batch_item`; add to `src/util/image/mod.rs`):

- `spawn_batch` `tokio::spawn`s a supervisor that builds the S3 client once (`aws_sdk_s3::Client::new(&state.aws_profile_picture_config)`, clone is Arc-cheap), creates `Semaphore::new(num_cpus::get().max(1))`, and drives items through a `JoinSet` (acquire `acquire_owned()` permit before each spawn).
- `process_batch_item` runs the body inside a panic-safe wrapper so a panic becomes `Failed{reason}` (no item left stuck): read temp file → `Encoding` → EXIF on `spawn_blocking` + `tokio::join!(process_uploaded_image(.., Photograph), process_uploaded_image(.., Thumbnail))` (reuse existing `src/util/image/process_uploaded_images.rs`) → `Uploading` → two S3 PUTs **preserving the existing orphan-cleanup** (delete main if thumb fails; delete both if DB insert fails, per `upload_photograph.rs:408-431,490-508`) → `Persisting` → acquire conn, insert `PhotographInsertable`, `drop(conn)` → `Completed{...}`. Delete the temp file in the terminal step. Bounded concurrency caps both blocking-pool pressure and simultaneous `get_conn()` calls.

**TTL/eviction** — new `src/jobs/maintenance/prune_photograph_batches.rs` + `supervise("PRUNE_PHOTOGRAPH_BATCHES", ...)` in `init_scheduler.rs`. `prune_terminal_batches` uses `retain_async`: drop terminal batches idle > 30 min, and any batch (stuck) idle > 6 h (hard cap); fire-and-forget temp-dir cleanup on drop. Add a startup sweep of stale `cyhdev-batch/*` dirs to clear leftovers from a previous process. Satisfies "no unbounded runtime-growing caches."

### Frontend

- **DTOs** — add to `src/dtos/responses/photography/index.ts`: `BatchItemStatusName` union, `BatchItemStatus`, `BatchUploadResponse`, `BatchStatusResponse`, `BatchListResponse`.
- **API client** — add to `photographyApi` in `src/services/all_api.ts`: `batchUpload(formData, {onUploadProgress})` (via `postFormData`), `getBatchStatus(batch_id)` and `getBatches()` (via `get`).
- **Store + poller** — new `src/state/photo_batches.ts` (module-scoped, survives route navigation, mirrors `src/state/auth.ts`). `createStore<Record<string, BatchEntry>>`; a single shared `setInterval` (~1.2 s) polling all active batches via `Promise.allSettled` with an `inFlight` guard against overlap; stops when no active batches remain; marks `_missing` on 404 (with a short grace for the first ticks after `trackBatch`); `_gridRefreshed` flag dedupes the grid reload; `activeBatchCount` memo for the badge; `setBatchCompletionHandler` so the page registers/clears its `fetchPhotos` reset on mount/cleanup.
- **Multi-file picker (per-file mini-forms)** — new `src/components/photographs/BatchUploadFields.tsx`. `<input multiple>`, collect `File[]`, render a row per file: object-URL thumbnail (revoke on remove/close), a comment input, and a location picker that reuses the existing Leaflet `UploadMap` targeting the active row. Per-file validation; cap batch size with an i18n error. Build FormData: ordered `files` + `meta` JSON array aligned to file order + `context`.
- **Processing button + popup** — new `src/components/photographs/ProcessingModal.tsx`. A "Processing" button beside Upload (gated on `isSuperuser()`) with an active-batch badge. Popup reuses `.modal-overlay`/`.modal-content` (+ a `.processing-modal` rule in the inline `<style>`); lists batches with a summary ("12/40 encoded, 1 failed"), an overall progress bar (`Math.min/Math.max`, guard `total===0`), and per-item status chips. Add chip tokens to `pageStyles.ts` (`chipBase`, `chipQueued`, `chipActive`, `chipCompleted`, `chipFailed`) with a `chipClass(status)` helper. `role="dialog" aria-modal` + Escape + `aria-live` summary.
- **Wire-up in `photographs.tsx`** — swap single input for `BatchUploadFields`, render `ProcessingModal`, rewrite `handleUpload` to call `batchUpload`, then `trackBatch(initial)` (seed a synthetic `queued` snapshot), close the upload modal, open Processing. On batch completion the store calls the registered handler → existing reset + `fetchPhotos()`. Do not hard-disable Upload during in-flight batches (concurrent batches are supported); only disable while a single upload request is in flight.
- **i18n** — add keys to all three i18n files: `photos.processing`, `photos.batch_summary` (interpolated via `tx`), `photos.batch_clear_finished`, `photos.batch_no_active`, `photos.select_images`, `photos.batch_too_many`, and `photos.status_{queued,encoding,uploading,persisting,completed,failed}`.

---

## Phase 2 — Views, votes, comments (separate tables, mirror blog)

Clone the blog system (`src/handlers/blog/`, `src/domain/blog/blog.rs`, `src/dtos/.../blog/`, `src/pages/posts/View.tsx`) onto photographs with separate tables. Vote/comment are **protected tier** (any authenticated user); reading is public; comment edit/delete is author-or-superuser. Views are naive +1 on detail open.

### Backend

- **Migration** (new dir under `rust-be-template/migrations/`, then regenerate `src/schema.rs`):
  - `ALTER TABLE photographs ADD COLUMN photograph_view_count int8 NOT NULL DEFAULT 0, ADD COLUMN photograph_total_upvotes int8 NOT NULL DEFAULT 0, ADD COLUMN photograph_total_downvotes int8 NOT NULL DEFAULT 0;` + btree indexes on each (DESC) for sortability.
  - `photograph_votes (photograph_vote_id uuid PK, photograph_id uuid FK, user_id uuid FK, photograph_vote_created_at timestamptz, is_upvote bool, UNIQUE(photograph_id, user_id))`.
  - `photograph_comments (photograph_comment_id uuid PK, photograph_id uuid FK, user_id uuid FK, photograph_comment_content text, photograph_comment_created_at timestamptz, photograph_comment_updated_at timestamptz NULL, parent_photograph_comment_id uuid NULL self-FK, photograph_comment_total_upvotes int8, photograph_comment_total_downvotes int8)` + index on `(photograph_id)`.
  - `photograph_comment_votes (photograph_comment_vote_id uuid PK, photograph_comment_id uuid FK, user_id uuid FK, created_at timestamptz, is_upvote bool, UNIQUE(photograph_comment_id, user_id))`.
- **Domain** — extend `Photograph` (add the three new columns) and add `PhotographVote`/`NewPhotographVote`, `PhotographComment`/`PhotographCommentResponse` (enriched with user badge + `VoteState`), `PhotographCommentVote`/`NewPhotographCommentVote` in `src/domain/photography/`. Reuse the existing `VoteState` enum from `src/domain/blog/blog.rs`.
- **Detail endpoint** — `GET /api/photographs/{photograph_id}` (public, mirror `read_post`): increment `photograph_view_count + 1` and return the photo + denormalized vote counts + caller's `vote_state` + threaded enriched comments. The frontend detail modal calls this on open (replacing reliance on stale list data).
- **Handlers** (mirror blog, protected tier): `vote_photograph`, `rescind_photograph_vote`, `vote_photograph_comment`, `rescind_photograph_comment_vote`, `submit_photograph_comment`, `update_photograph_comment` (author/superuser), `delete_photograph_comment` (author/superuser, hard delete, FK cascade). Same upsert-`ON CONFLICT`-then-recount-and-update-denormalized-columns pattern as `vote_post.rs`. Request/response DTOs mirror `{ is_upvote }` and `{ upvote_count, downvote_count, is_upvote }`.
- **Routes** — register the vote/comment routes in `protected_router` and the detail GET in `public_router` in `main_router.rs`.

### Frontend

- **DTOs/client** — add request/response DTOs under `src/dtos/.../photography/` and `photographyApi` methods: `getPhotographDetail`, `votePhotograph`, `rescindPhotographVote`, `votePhotographComment`, `rescindPhotographCommentVote`, `submitPhotographComment`, `updatePhotographComment`, `deletePhotographComment` (mirror `blogApi`).
- **Detail modal** — in `photographs.tsx` (~line 950, after the comments heading) add, matching blog visual style: a view-count line, the ▲/net-score/▼ vote control (emerald active up / rose active down, same as `posts/View.tsx`), a threaded comment list with per-comment vote buttons, a comment composer (textarea + `pageStyles.buttonPrimary`), and author/superuser edit/delete controls. Fetch detail via `getPhotographDetail` on open; optimistic vote store keyed by `photograph_id` with rollback (mirror `View.tsx`). Extract comment tree rendering into a component to respect the 300-LOC guideline (`photographs.tsx` is already ~1064 lines).
- **i18n** — add comment/vote/view keys to all three i18n files.

---

## Key risks / invariants (baked into the design)

- **404-not-403 for batch privacy** — otherwise the JSON poller triggers the global logout/redirect. (Same applies to the photograph detail endpoint: keep it public/200 like `read_post`.)
- **Memory** — temp-file staging + `Semaphore(num_cpus)` keep resident bytes at roughly `permits × file_size`, not `N × 150 MB`.
- **No stuck items** — panic-safe wrapper + idempotent terminal counters + hard-TTL guarantee every batch eventually becomes `done` and is evicted.
- **In-memory tracker is ephemeral** — a server restart loses batch tracking (acceptable, documented); rows/objects that already reached Persisting/Completed survive; startup sweep clears orphaned temp dirs.
- **Poller hygiene** — single shared interval, `inFlight` guard, stop-on-empty, `onCleanup` clears the completion handler to avoid stale closures over an unmounted page.
- **i18n is compile-blocking** — every new key in `keys.ts` + both default bundles.

## Verification

Backend:
- `cargo fmt && cargo clippy` clean. Run the Diesel migration; confirm `schema.rs` is regenerated and indexes exist.
- Batch upload: `curl -F context=photography -F 'files=@a.jpg' -F 'files=@b.jpg' -F 'meta=[{"comment":"x","lat":1,"lon":2},{"comment":"y","lat":3,"lon":4}]'` to `/api/photographs/batch-upload` (with superuser session cookie) returns 202 + `batch_id` immediately; poll `GET /api/photographs/batch/{id}` and watch items move queued→encoding→…→completed; rows appear in `photographs`, objects in S3.
- Privacy: poll a random/other-user `batch_id` → 404 (never 403). Force a bad image to confirm `Failed{reason}` and that the batch still reaches `done`. Confirm the prune job evicts terminal batches after TTL (temporarily lower TTL to verify).
- Social: vote/unvote a photo and a comment (toggle up/down, denormalized counts update); submit/edit/delete threaded comments (author + superuser paths, 401 for guests); open detail → `photograph_view_count` increments by 1 per call.

Frontend:
- `npm run build` / `tsc` clean (all i18n keys present). Run the SPA (`/run` or `npm run dev`).
- Select many files in the upload modal, fill per-file comment + location, submit → modal closes, Processing badge shows count, popup shows per-image progress; on completion the grid refreshes. Navigate away and back → polling/badge persist. Reload mid-batch → poll 404 handled gracefully (no logout).
- Open a photo → view count shows/increments, vote ▲/▼ reflects state with optimistic update + rollback on forced failure, comments thread/edit/delete render in the same visual style as the blog.
