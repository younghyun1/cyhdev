# KCNA archive backend feature

`rust-be-template/src/features/kcna_archive` serves the [archive snapshot](kcna-archive-serving.md) to superusers and to the accounts a superuser lists. The snapshot is the data source for every read, so no read touches PostgreSQL. The slice has `api`, `service`, `domain`, `repository`, and `error.rs`; the domain and repository hold one thing, the reader list, and `composition.rs` wires the service to its store. The service calls `kcna-archive-core`, whose result models act as the domain types; the API converts them explicitly to its own camelCase contracts and derives no schema on core types.

## Who may read

The corpus is public text from a news agency that is not released under this site's license, so it is not redistributable, and nobody but superusers reads the archive unless a superuser lists their account. The list is per account: there is no setting that opens the archive to every member. A listed account still has to be signed in with a verified address. Being listed grants reading only: the snapshot's status, its reload, and the list itself stay with superusers.

The reader routes sit behind the session middleware, which already rejects an account without a verified address, and then behind the archive's own gate (`api/guard.rs`), which admits a superuser or an account on the list. The gate reads a copy of the list held in memory, so no request waits on PostgreSQL. The stored list is the table `kcna_archive_reader`: the account's identifier as primary key, referencing `users`, and the time it was listed. The list holds at most 500 accounts; it is kept whole in memory and shown on one page.

The copy in memory is replaced from the store when the service starts, when a superuser reads the list, and after each change, in that order under one lock, and the backend is one process, so nothing else can leave it stale. A grant or removal is applied to the copy as soon as it is stored. A failed read at startup leaves the copy empty, which admits nobody, and a failed change leaves the list as it was. Each change is logged with the acting account.

Only an active account can be listed: one that exists and is neither deleted, purged, nor a system actor. The check and the insert run in one transaction with a share lock on the account row, which conflicts with the lock that deletion takes. A row whose account is deleted later is skipped when the list is read, so the account drops off the list and out of memory; the row itself is left for the accounts feature's retention rules, and a deleted account cannot sign in anyway.

Every archive response carries `Cache-Control: no-store`: the reader routes through their own layer, which also covers the session check's rejections, and the administrator routes through the `/api/admin/` policy. The export is a file download but passes through the same guards.

## Routes

Reader routes are under `/api/kcna-archive` and administrator routes under `/api/admin/kcna-archive`. Every one answers 401 without a session. A reader route answers 403 with code 119 to an account that is neither a superuser nor listed; an administrator route answers 403 to a non-superuser. An account without a verified address gets the session middleware's 400 (code 41) on all of them. JSON responses use the usual `{ success, data, meta }` envelope, and errors use the common error body.

| Method and path | For | Purpose |
| --- | --- | --- |
| `GET /api/kcna-archive/search` | Readers | One page of results with highlighted snippets and facet counts. |
| `GET /api/kcna-archive/histogram` | Readers | Matches per year, month, or day bucket. |
| `GET /api/kcna-archive/trends` | Readers | Counts for one to four repeated `term` values per bucket. |
| `GET /api/kcna-archive/documents/{document_id}` | Readers | One document with optional `q` highlighting. |
| `GET /api/kcna-archive/export` | Readers | Up to 10,000 results as `csv` (default) or `jsonl`, sent as an attachment. |
| `GET /api/kcna-archive/access` | Any verified account | Whether the caller may read (`canRead`). |
| `GET /api/admin/kcna-archive/status` | Superusers | Service state and, when a snapshot is loaded, its corpus summary. |
| `POST /api/admin/kcna-archive/reload` | Superusers | Reads the snapshot from disk again and swaps it in. |
| `GET /api/admin/kcna-archive/readers` | Superusers | The reader list, newest grant first, with each account's name and address. |
| `POST /api/admin/kcna-archive/readers` | Superusers | Lists the account in `userId`; answers with the list. |
| `DELETE /api/admin/kcna-archive/readers/{user_id}` | Superusers | Removes the account; answers with the list. |

`GET /api/kcna-archive/access` answers 200 to an account it turns away, because the site asks it to decide whether to show the archive at all. Its `adminOnly` field is true only for a superuser while nobody is listed, so the page can mark the archive as theirs alone; it is false for every other caller, who therefore learns nothing about the list. `POST /reload`, `POST /readers`, and `DELETE /readers/{user_id}` are state-changing and also need a trusted browser origin, like the other administrative writes. Listing an account twice and removing one that is not listed both answer 200 and change nothing. A change applies to the account's next request.

## States

The service holds one of four states. `not_installed` means `snapshot.json` is absent, which is normal on machines without the corpus and logs one info line. `loading` lasts while the first load runs. `ready` holds the snapshot. `failed` means the first load failed; the cause is logged once with structured fields. Startup never waits for the load: the composition root starts it on the blocking pool and the server accepts connections meanwhile.

`GET /status` always answers 200 with a `state` field, so the page can show a missing or failing archive without treating it as a transport error; `snapshot` is present only in `ready`, and `reloading` and `lastReloadFailed` describe reloads. The data routes answer 503 while not ready, with distinct error codes: 113 not installed, 114 loading (with `Retry-After`), 115 failed.

## Reload

`POST /reload` loads the directory on the blocking pool and replaces the snapshot atomically. The old snapshot keeps answering until the new one is ready, so memory holds about two snapshots during a reload. Only one load runs at a time, including the first; a second request gets 409 (code 116). A failed reload keeps the old snapshot, sets `lastReloadFailed`, and answers 500 (code 117). A reload from `not_installed` or `failed` installs a snapshot copied after startup. The reload runs on a detached task, so a client that disconnects cannot release the single-load gate while the blocking load continues.

## Bounds

Queries run on the blocking pool under a 64-permit semaphore. That is well above what the listed readers produce, so they do not queue behind one another, and it still stops a runaway burst from taking every thread of the pool shared with image and password work. It is more than the server has cores, so it bounds threads, not processor time. The permit moves into the blocking task and is held until the work ends. The core crate bounds the rest: query text of 200 characters, pages of 100 results, a result window of 10,000, at most four trend terms, and 10,000 export rows. The `term` list is read one past the limit so an oversized request is rejected rather than truncated. Request bodies are capped at 1 KiB: reads take their parameters in the query string and the one body is a single account identifier. Listed accounts share the semaphore with superusers; there is no per-account limit, because the list holds only people the owner chose.

## Errors

| Condition | Status | Code |
| --- | --- | --- |
| Malformed parameter or id | 400 | 45 |
| Document not in the snapshot | 404 | 112 |
| Snapshot not installed, loading, failed | 503 | 113, 114, 115 |
| Reload already running | 409 | 116 |
| Reload failed, old snapshot kept | 500 | 117 |
| Index or task failure | 500 | 118 |
| Account is neither a superuser nor listed | 403 | 119 |
| Reader list could not be read or changed | 500 | 120 |
| No active account to list | 404 | 121 |
| Reader list is full | 409 | 122 |

## Configuration

`KCNA_ARCHIVE_DIR` names the snapshot directory and defaults to `data/kcna_archive`, relative to the backend working directory. The container image has an empty read-only mount point at that path, `/bin/data/kcna_archive`; mount the snapshot there or set the variable. The directory must be readable by the unprivileged runtime user.
