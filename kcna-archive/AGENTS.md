# KCNA archive

Two workspace crates. `core/` (`kcna-archive-core`) is what the website links: index layout, analyzer, snapshot loader, and the read-only service. `cli/` (`kcna-archive`) is the collector, the PostgreSQL importer, and the snapshot writer. Read [archive serving](../docs/architecture/be/kcna-archive-serving.md) before changing either, and [collection](../docs/architecture/be/kcna-archive-collection.md) or [corpus storage](../docs/architecture/be/kcna-archive-corpus.md) for the collector. The backend slice is `rust-be-template/src/features/kcna_archive/` and the browser is `solid-csr-spa-template/src/pages/kcna_archive/`; their own scoped instructions apply there.

## Boundaries

- `core` has no database, HTTP, or async runtime dependency, and must stay that way: the backend calls it from its blocking pool. The corpus enums map to PostgreSQL only behind its `diesel` feature, which only `cli` enables.
- The website never reads the collector's database. Anything a reader needs has to be in the snapshot. Changing the index fields or storage settings in `core/src/search/fields.rs` changes the snapshot format: existing snapshots are rebuilt on the next run, and the server and the snapshot must be deployed from the same revision.
- `search_text.rs` is the single analyzer for indexing, querying, and highlighting. Do not add a second tokenization path.
- The collector's migrations in `cli/migrations/` belong to the corpus database, not to the website's. Keep `cli/src/database/schema.rs` in step with them and load the database conventions before changing either.
- The corpus (`data/`, snapshots) is not redistributable and stays out of Git, the backend binary, and container images. Archive routes stay superuser-only.

## Commands

From the repository root: `cargo test --package kcna-archive-core --package kcna-archive` for both crates; `KCNA_TEST_DATABASE_URL` naming a disposable PostgreSQL 18 database additionally enables the collector's round-trip test. `cargo xtask kcna-snapshot` writes the snapshot the development backend loads, `cargo run --package kcna-archive -- --snapshot-dir <dir> verify` loads one as the server does and times requests, and `cargo xtask kcna-snapshot-push <rsync destination>` copies it to a server. Root `cargo xtask clippy`, `fmt`, and `unit` cover both crates. Collection commands contact the source sites; do not run them as part of verification.
