# November 21 KCNA archive integration and three more languages

Status: complete locally on November 21, 2026; not deployed and not pushed. The KCNA research archive, previously a standalone project, is part of this workspace and is served by the site from memory to administrator accounts. The interface gained Dutch, Ukrainian, and Turkish.

Checkout: `main`. The work is `445b378` (languages and archive strings), `3c6bd1d` (archive crates, backend feature, snapshot tooling), `e7fb4c7` (site section), and the two commits after it (training campaign coverage, then this plan). Nothing is pushed. The collector came from a private repository (`kcna-archive`, branch `redesign/search-first` at `c67b9b8`) and this repository is public, so the first push publishes the collector's source; the corpus itself stays out of Git.

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
- Server: `cargo xtask kcna-snapshot-push miniserver:dist/data/kcna_archive/` copied 21 files in 6 s; every file's SHA-256 matches the local snapshot.

Not run: any release or optimized build, the live training campaign (its archive scenario, fixture reset, and page assertions were checked by unit tests and by reading only, so the first real campaign run exercises them for the first time), WebKit, the performance and security browser suites, the migration rollback suite (no site migration changed), and the backend against a live database. The archive pages were exercised against mocked responses and the routes against a synthetic snapshot; the real snapshot was loaded and queried through the core service and the backend service, not through HTTP with a signed-in session.

## Remaining

1. Deploy: build the optimized artifact through the normal workflow, install it at `miniserver:~/dist`, and restart the service. The snapshot is already at `~/dist/data/kcna_archive`, which is the default location for a server whose working directory is `~/dist`; no environment change is needed. The snapshot and the server must come from the same revision, because the index layout is part of the format.
2. After the restart, sign in as a superuser, open `/projects/kcna-archive/status`, and confirm 399,902 searchable articles and the snapshot time. This is the first end-to-end check through HTTP.
3. Decide whether to push, given that it publishes the collector's source.
4. The corpus (`data/`, 3.9 GB) and the corpus database container are still where the standalone project left them. `cargo xtask kcna-snapshot` reads `DATABASE_URL` and needs no corpus directory; collection and import take `--data-dir`. Moving the corpus to `kcna-archive/data` (ignored by Git) is optional.
5. Native-speaker review of the Dutch, Ukrainian, and Turkish catalogs and About sections, and of the archive strings in the other 24 non-English catalogs, as noted in [localization](../design/fe/localization.md).
6. To refresh the archive later: collect, `database import`, `cargo xtask kcna-snapshot-push`, then Reload snapshot on the status page.
