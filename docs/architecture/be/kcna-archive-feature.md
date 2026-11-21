# KCNA archive backend feature

`rust-be-template/src/features/kcna_archive` serves the [archive snapshot](kcna-archive-serving.md) to superuser accounts. The snapshot is the data source, so the slice has `api`, `service`, and `error.rs` but no repository layer and no PostgreSQL access. The service calls `kcna-archive-core`, whose result models act as the domain types; the API converts them explicitly to its own camelCase contracts and derives no schema on core types.

## Why administrators only

The corpus is public text from a news agency that is not released under this site's license, so it is not redistributable. Every route sits in the superuser router, behind the session and role middleware, and every `/api/admin/` response carries `Cache-Control: no-store`. The export is a file download but passes through the same guards.

## Routes

All routes are under `/api/admin/kcna-archive`. Every one answers 401 without a session and 403 for a non-superuser. JSON responses use the usual `{ success, data, meta }` envelope, and errors use the common error body.

| Method and path | Purpose |
| --- | --- |
| `GET /search` | One page of results with highlighted snippets and facet counts. |
| `GET /histogram` | Matches per year, month, or day bucket. |
| `GET /trends` | Counts for one to four repeated `term` values per bucket. |
| `GET /status` | Service state and, when a snapshot is loaded, its corpus summary. |
| `GET /documents/{document_id}` | One document with optional `q` highlighting. |
| `GET /export` | Up to 10,000 results as `csv` (default) or `jsonl`, sent as an attachment. |
| `POST /reload` | Reads the snapshot from disk again and swaps it in. |

`POST /reload` is state-changing and also needs a trusted browser origin, like the other administrative writes. The export is a deliberate non-JSON response: its body is the file, with `Content-Disposition: attachment` and `X-Content-Type-Options: nosniff`, and it has no generated client function.

## States

The service holds one of four states. `not_installed` means `snapshot.json` is absent, which is normal on machines without the corpus and logs one info line. `loading` lasts while the first load runs. `ready` holds the snapshot. `failed` means the first load failed; the cause is logged once with structured fields. Startup never waits for the load: the composition root starts it on the blocking pool and the server accepts connections meanwhile.

`GET /status` always answers 200 with a `state` field, so the page can show a missing or failing archive without treating it as a transport error; `snapshot` is present only in `ready`, and `reloading` and `lastReloadFailed` describe reloads. The data routes answer 503 while not ready, with distinct error codes: 113 not installed, 114 loading (with `Retry-After`), 115 failed.

## Reload

`POST /reload` loads the directory on the blocking pool and replaces the snapshot atomically. The old snapshot keeps answering until the new one is ready, so memory holds about two snapshots during a reload. Only one load runs at a time, including the first; a second request gets 409 (code 116). A failed reload keeps the old snapshot, sets `lastReloadFailed`, and answers 500 (code 117). A reload from `not_installed` or `failed` installs a snapshot copied after startup. The reload runs on a detached task, so a client that disconnects cannot release the single-load gate while the blocking load continues.

## Bounds

Queries run on the blocking pool under a four-permit semaphore, so one administrator's burst cannot occupy the pool shared with image and password work. The permit moves into the blocking task and is held until the work ends. The core crate bounds the rest: query text of 200 characters, pages of 100 results, a result window of 10,000, at most four trend terms, and 10,000 export rows. The `term` list is read one past the limit so an oversized request is rejected rather than truncated. Request bodies are capped at 1 KiB because every parameter travels in the query string.

## Errors

| Condition | Status | Code |
| --- | --- | --- |
| Malformed parameter or id | 400 | 45 |
| Document not in the snapshot | 404 | 112 |
| Snapshot not installed, loading, failed | 503 | 113, 114, 115 |
| Reload already running | 409 | 116 |
| Reload failed, old snapshot kept | 500 | 117 |
| Index or task failure | 500 | 118 |

## Configuration

`KCNA_ARCHIVE_DIR` names the snapshot directory and defaults to `data/kcna_archive`, relative to the backend working directory. The container image has an empty read-only mount point at that path, `/bin/data/kcna_archive`; mount the snapshot there or set the variable. The directory must be readable by the unprivileged runtime user.
