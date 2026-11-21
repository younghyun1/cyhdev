# November 21 KCNA archive integration and three more languages

Status: complete locally on November 21, 2026; not deployed and not pushed. The KCNA research archive, previously a standalone project, is part of this workspace and is served by the site from memory to administrator accounts. The interface gained Dutch, Ukrainian, and Turkish.

Checkout: `main`. The work is `445b378` (languages and archive strings), `3c6bd1d` (archive crates, backend feature, snapshot tooling), `e7fb4c7` (site section), and the commits after it (training campaign coverage, this plan, and the fixes from the live development run). Nothing is pushed. The collector came from a private repository (`kcna-archive`, branch `redesign/search-first` at `c67b9b8`) and this repository is public, so the first push publishes the collector's source; the corpus itself stays out of Git.

## Completed

- **Crates:** `kcna-archive/core` (index layout, analyzer, snapshot loader, read-only service; no database, HTTP, or async dependency) and `kcna-archive/cli` (collector, PostgreSQL importer, snapshot writer, `verify`). The standalone web server, its embedded frontend, and its build script are gone.
- **Snapshot:** one Tantivy index plus `snapshot.json`. It carries everything the reader needs, so the site asks no database for the archive. Stored text is Zstandard level 22 in 64 KiB blocks. Raw response bytes stay in the collector's database; the reader links the article at its source. See [archive serving](../architecture/be/kcna-archive-serving.md).
- **Backend:** `features/kcna_archive`, routes under `/api/admin/kcna-archive` behind the superuser check, loaded into memory at startup off the request path, with a reload that swaps a newer snapshot in. `KCNA_ARCHIVE_DIR` defaults to `data/kcna_archive` relative to the backend working directory; a machine without it reports the archive as not installed. See [the backend feature](../architecture/be/kcna-archive-feature.md).
- **Frontend:** four superuser routes under `/projects/kcna-archive`, a Projects card and admin navigation entry that render only for superusers, the site's design tokens, theme, and language selection, and 93 `kcna.*` strings in every catalog. See [the browser design](../design/fe/kcna-archive-browser.md).
- **Languages:** `nl-NL`, `uk-UA`, and `tr-TR`, each with a full catalog and both About sections, for 27 languages and 831 keys. Turkish was chosen as the third because it was the largest language still missing.
- **Tooling:** `cargo xtask kcna-snapshot` and `cargo xtask kcna-snapshot-push`; `cargo xtask fmt` covers both archive crates; search, compression, hashing, and HTML parsing crates are optimized in the development profile.
- **Training campaign:** the optional PGO and BOLT campaign covers the archive: four pages and the scenario `kcna.search-read-trends-reload` in `tools/optimization/coverage.json`, a deterministic synthetic snapshot of 24 invented documents written by the fixture tool on every reset, and request steps for all seven operations.

## Decisions

- **Snapshot instead of an in-memory repository.** The original in-memory mode deserialized every document into the heap and cost 3.1 GB. Holding the compressed index in memory costs what the index weighs on disk (655 MB, 731 MB resident) and needs no second representation.
- **Data outside the binary.** The snapshot is copied to the server on its own, so a new corpus needs no build and a new build needs no corpus transfer. A full rebuild takes 41 s, an unchanged corpus 0.8 s, and the copy to the server 6 s.
- **Feature unification.** The collector's Diesel dependency does not enable `without-deprecated`, because Cargo unifies features across a workspace build and that one removes items the backend imports.
- **Flat routes.** The route guards drop child props, so each archive page renders the shared section layout itself instead of nesting under a layout route.

## Verification

Observed on November 21, 2026, in development builds:

- `cargo xtask fmt`, `cargo xtask clippy` (native and WebAssembly), `cargo xtask openapi`, and `cargo xtask secret-scan`: exit 0.
- `cargo xtask unit`: exit 0; 656 passed across the workspace, including backend library 408, collector 66, core 39, xtask 86, and the fixture tool 15.
- `cargo xtask db-integration` against a disposable loopback PostgreSQL 18: 50 passed, including the 27-catalog synchronization.
- Collector round trip against a disposable PostgreSQL 18 (`KCNA_TEST_DATABASE_URL`): passed.
- `cargo xtask image-smoke`: exit 0 after the workspace gained the archive crates. The container was not run.
- `cargo xtask frontend-check`: exit 0; typecheck, lint with zero warnings, 506 unit tests, the 17 optimization driver tests, and the Vite build. Separately, 207 Chromium tests (archive, page renders, localization) and 185 layout-stability tests in every language pass against mocked responses.
- Full corpus: `database snapshot --rebuild` wrote 399,902 articles from 409,584 stored documents in 41 s; `verify` loaded 686 MB in 0.05 s with 731 MB resident and answered result pages in 1 to 16 ms.
- Live serving in a development build: the backend ran in a disposable runtime (the campaign's fixture tool and a disposable loopback PostgreSQL 18) with the real snapshot, over HTTPS. Anonymous requests got 401 and a signed-in member 403; a superuser got status (399,902 articles, 686 MB held), searches in 7 to 18 ms end to end, a document with its five other language editions, a 29-bucket year histogram, two-term trends, a CSV export with `no-store` and attachment headers, and a reload in 0.07 s. The page shell was served from the built frontend.
- Restart in that build: graceful stop 0.14 s, restart to healthy 18.5 s, archive ready 0.2 s later. The snapshot load is 0.05 s of that and happens off the startup path; 18.05 s is the Geo-IP database load in an unoptimized build, which predates this work and does not reflect an optimized build.
- Production restart baseline, read from the server's own log on November 21 without touching the service: the current optimized build initializes in 5.96 s, of which 4.79 s is the Geo-IP load. The archive adds its 0.05 s snapshot load off the startup path, so a restart after deployment should take about the same six seconds.
- Training campaign in development diagnostic mode against the same kind of runtime, with the synthetic snapshot: the archive scenario passed and covered all seven operations including its browser steps, and the page census passed for all 39 pages. This run found and fixed two defects the unit checks could not see: fixture counts written as numbers where the driver admits only strings, and a JSON Lines export handed to a step that parses one JSON document. The `kcna` diagnostic group was added for this.
- Server: `cargo xtask kcna-snapshot-push miniserver:dist/data/kcna_archive/` copied 21 files in 6 s; every file's SHA-256 matches the local snapshot.

Not run: any release or optimized build, the full training campaign in an optimized stage (only its development diagnostics), WebKit, the performance and security browser suites, the migration rollback suite (no site migration changed), and anything against the production database or the running server. The archive pages were rendered against the real snapshot only through the API checks above; the browser runs used mocked responses or the synthetic snapshot.

## Remaining

1. Deploy: build the optimized artifact through the normal workflow, install it at `miniserver:~/dist`, and restart the service. This needs the owner: release builds are outside what repository instructions allow during implementation work, and the service runs under OpenRC as root, so the restart needs sudo. The snapshot is already at `~/dist/data/kcna_archive`, which is the default location for a server whose working directory is `~/dist`; no environment change is needed. The snapshot and the server must come from the same revision, because the index layout is part of the format.
2. After the restart, sign in as a superuser, open `/projects/kcna-archive/status`, and confirm 399,902 searchable articles and the snapshot time. This is the first check on the server itself; the same check passed in a development build against a disposable runtime.
3. Decide whether to push, given that it publishes the collector's source.
4. The corpus (`data/`, 3.9 GB) and the corpus database container are still where the standalone project left them. `cargo xtask kcna-snapshot` reads `DATABASE_URL` and needs no corpus directory; collection and import take `--data-dir`. Moving the corpus to `kcna-archive/data` (ignored by Git) is optional.
5. Native-speaker review of the Dutch, Ukrainian, and Turkish catalogs and About sections, and of the archive strings in the other 24 non-English catalogs, as noted in [localization](../design/fe/localization.md).
6. To refresh the archive later: collect, `database import`, `cargo xtask kcna-snapshot-push`, then Reload snapshot on the status page.
