# November 21 KCNA archive integration and three more languages

Status: the first version is pushed and deployed; the access follow-up of the same day is committed locally and not deployed. The KCNA research archive, previously a standalone project, is part of this workspace and is served by the site from memory. Administrators can always read it; other members can only if an administrator lists their account. The interface gained Dutch, Ukrainian, and Turkish.

Checkout: `main`. The first version is `445b378` through `ebaa413` and is on `origin/main`; the server's process started on November 21 at 14:57 MST from a binary installed a minute earlier, which is that version. The follow-up is the commits after `ebaa413`: a wider layout that stays where it is when a result opens, and the per-account reader list with its access page, Projects menu entry, Not Found answers, and the maintainer's disclaimer. The collector came from a private repository (`kcna-archive`, branch `redesign/search-first` at `c67b9b8`); the corpus itself stays out of Git.

## Completed

- **Crates:** `kcna-archive/core` (index layout, analyzer, snapshot loader, read-only service; no database, HTTP, or async dependency) and `kcna-archive/cli` (collector, PostgreSQL importer, snapshot writer, `verify`). The standalone web server, its embedded frontend, and its build script are gone.
- **Snapshot:** one Tantivy index plus `snapshot.json`. It carries everything the reader needs, so the site asks no database for the archive. Stored text is Zstandard level 22 in 64 KiB blocks. Raw response bytes stay in the collector's database; the reader links the article at its source. See [archive serving](../architecture/be/kcna-archive-serving.md).
- **Backend:** `features/kcna_archive`. Reading is under `/api/kcna-archive` behind the session check and the archive's own gate; the snapshot's status, its reload, and the reader list are under `/api/admin/kcna-archive` behind the superuser check. The snapshot loads into memory at startup off the request path, with a reload that swaps a newer one in. `KCNA_ARCHIVE_DIR` defaults to `data/kcna_archive` relative to the backend working directory; a machine without it reports the archive as not installed. See [the backend feature](../architecture/be/kcna-archive-feature.md).
- **Reader list:** the accounts allowed to read, one row each in `kcna_archive_reader`, added by migration `2026-11-21-000001_kcna_archive_access`. The list is empty on a new installation, holds at most 500 accounts, and only active accounts can be added. The gate reads a copy in memory, so no read waits on PostgreSQL. A superuser manages it at `/admin/kcna-archive`.
- **Frontend:** search, reader, and trends under `/projects/kcna-archive` for superusers and listed members, the status page for superusers, and the access page in the admin workspace, where a superuser finds members by name or address and allows or removes them. The archive is listed in the Projects menu, on the Projects page, and in the command palette only for a visitor who may read it; anyone else, signed in or not, gets the Not Found page in place. The section uses the site's design tokens, theme, and language selection, is up to 108rem wide, and keeps the page where it is when a result opens beside the list. Every archive page carries the maintainer's disclaimer. There are 104 `kcna.*` strings in every catalog. See [the browser design](../design/fe/kcna-archive-browser.md).
- **Languages:** `nl-NL`, `uk-UA`, and `tr-TR`, each with a full catalog and both About sections, for 27 languages and, with the follow-up's eleven strings, 842 keys. Turkish was chosen as the third because it was the largest language still missing.
- **Tooling:** `cargo xtask kcna-snapshot` and `cargo xtask kcna-snapshot-push`; `cargo xtask fmt` covers both archive crates; search, compression, hashing, and HTML parsing crates are optimized in the development profile.
- **Training campaign:** the optional PGO and BOLT campaign covers the archive: five pages and the scenario `kcna.search-read-trends-reload` in `tools/optimization/coverage.json`, a deterministic synthetic snapshot of 24 invented documents written by the fixture tool on every reset, and request steps for all eleven operations. The scenario lists the fixture member, reads as that member, and removes them again, so each stage starts with an empty list.

## Decisions

- **Snapshot instead of an in-memory repository.** The original in-memory mode deserialized every document into the heap and cost 3.1 GB. Holding the compressed index in memory costs what the index weighs on disk (655 MB, 731 MB resident) and needs no second representation.
- **Data outside the binary.** The snapshot is copied to the server on its own, so a new corpus needs no build and a new build needs no corpus transfer. A full rebuild takes 41 s, an unchanged corpus 0.8 s, and the copy to the server 6 s.
- **Feature unification.** The collector's Diesel dependency does not enable `without-deprecated`, because Cargo unifies features across a workspace build and that one removes items the backend imports.
- **A list of accounts, not a switch or a permission binding.** Access is granted one account at a time; a setting that opened the archive to every verified member was built first and replaced the same day at the owner's direction. The site's role and permission catalog was not used either: its checks read PostgreSQL on every use, and the archive is searched as the reader types, so its gate reads a copy of the list in memory. A failed read at startup admits nobody.
- **Not Found instead of sign-in.** A visitor who may not read the archive gets the site's Not Found page at the address they asked for. The archive is not advertised to someone who cannot open it, and the answer does not depend on whether they are signed in.
- **Reading moved out of `/api/admin`.** Listed members read, so the reader routes are `/api/kcna-archive`; they keep `no-store` through their own layer. Only the status, reload, and reader-list routes remain administrative.
- **Flat routes.** The route guards drop child props, so each archive page renders the shared section layout itself instead of nesting under a layout route.

## Verification

Observed on November 21, 2026, in development builds, on the tree with the follow-up:

- `cargo xtask fmt`, `cargo xtask clippy` (native and WebAssembly), `cargo xtask openapi`, and `cargo xtask secret-scan`: exit 0.
- `cargo xtask unit`: exit 0; 665 passed across the workspace, including backend library 416 (31 of them the archive's, with the gate, the reader list service, and the routes behind the real guards), collector 66, core 39, xtask 86, and the fixture tool 16.
- `cargo xtask db-integration` against a disposable loopback PostgreSQL 18: 51 passed, including the new `kcna_archive_access` case (the reader list against real accounts) and the 27-catalog synchronization. `cargo xtask migration-rollback` against the same server: exit 0.
- `cargo xtask frontend-check`: exit 0; typecheck, lint with zero warnings, 521 unit tests, the 17 optimization driver tests, and the Vite build.
- Browser tests in Chromium against mocked responses: 207 passed across the archive, page-render, localization, and admin specs (the archive spec was also repeated three times without a failure), and 189 layout-stability tests in every language, including the access page.

Observed earlier the same day on the first version, and not repeated for the follow-up:

- Collector round trip against a disposable PostgreSQL 18 (`KCNA_TEST_DATABASE_URL`): passed. The collector is unchanged since.
- `cargo xtask image-smoke`: exit 0. The container was not run.
- Full corpus: `database snapshot --rebuild` wrote 399,902 articles from 409,584 stored documents in 41 s; `verify` loaded 686 MB in 0.05 s with 731 MB resident and answered result pages in 1 to 16 ms. The snapshot format is unchanged, so the snapshot on the server stays valid.
- Live serving in a development build with the real snapshot over HTTPS, in a disposable runtime: status, searches in 7 to 18 ms end to end, a document with its five other language editions, histogram, trends, export, and a reload in 0.07 s. Restart to healthy took 18.5 s, of which 18.05 s is the Geo-IP load in an unoptimized build; the server's own log shows 5.96 s for an optimized build.
- Training campaign in development diagnostic mode: the archive scenario and the page census passed. The follow-up changed the scenario and added a page; only the campaign's static checks were run on that.
- Server: `cargo xtask kcna-snapshot-push miniserver:dist/data/kcna_archive/` copied 21 files in 6 s; every file's SHA-256 matched.

Not run: any release or optimized build, the training campaign against a live backend after the follow-up, WebKit, the performance and security browser suites, and anything against the production database or the running server. The reader list was exercised against PostgreSQL through the integration case and against the HTTP guards through the route tests, but not through a running backend with a browser.

## Remaining

1. Deploy the follow-up: `./build.sh`, copy `target/x86_64-unknown-linux-gnu/release/rust-be-template` to `miniserver:~/rust-be-template`, compare SHA-256, then `ssh -t miniserver ./deploy.sh`. This needs the owner: release builds are outside what repository instructions allow during implementation work, and the restart needs sudo. The new migration applies at startup. The snapshot needs no new copy. The frontend and the API paths changed together and ship in one binary.
2. After the restart, sign in as a superuser and check `/projects/kcna-archive` from the Projects menu, then `/admin/kcna-archive`. The list starts empty; allowing a member there is the first time it is written on the server. Check as a member who is not listed, or signed out, that the archive answers Not Found.
3. Decide whether to push the follow-up.
4. The corpus (`data/`, 3.9 GB) and the corpus database container are still where the standalone project left them. `cargo xtask kcna-snapshot` reads `DATABASE_URL` and needs no corpus directory; collection and import take `--data-dir`. Moving the corpus to `kcna-archive/data` (ignored by Git) is optional.
5. Native-speaker review of the Dutch, Ukrainian, and Turkish catalogs and About sections, of the archive strings in the other 24 non-English catalogs, and of the disclaimer in all 26, as noted in [localization](../design/fe/localization.md).
6. To refresh the archive later: collect, `database import`, `cargo xtask kcna-snapshot-push`, then Reload snapshot on the status page.
7. Listed members share the four-permit query limit with administrators. There is no per-account limit, because the list holds only people the owner chose; add one if that changes.
