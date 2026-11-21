# KCNA archive serving

The website serves the Korean Central News Agency research archive from memory. The collector keeps PostgreSQL as the store of record on the workstation; the server never sees that database. What travels to the server is a snapshot: one Tantivy index that holds everything a reader needs, plus a small manifest. The backend reads the snapshot into process memory and answers search, reading, histograms, trends, exports, and status from it without touching the filesystem or a database. The corpus text is not redistributable, so the archive is reachable by superuser accounts only; see [the backend feature](kcna-archive-feature.md) for routes, states, and limits.

An earlier design kept a document store in PostgreSQL beside an on-disk index, and before that an in-memory repository of deserialized documents that cost 3.1 GB of heap. The snapshot replaces both: it needs no database at request time and occupies about the same memory as its files on disk.

## Crates

`kcna-archive/core` owns the index layout, the analyzer shared by indexing, querying, and highlighting, the snapshot loader, and the read-only service. It has no database, HTTP, or async runtime dependency; every service method is synchronous and CPU-bound, and the backend calls it from the blocking pool. `kcna-archive/cli` is the collector: it downloads pages, imports them into PostgreSQL, and writes the snapshot. The corpus enums live in the core crate, which maps them to their PostgreSQL types only behind its `diesel` feature, so the backend links no archive table code. See [collection](kcna-archive-collection.md) and [corpus storage](kcna-archive-corpus.md).

## Snapshot

A snapshot is a directory with `snapshot.json` and the index in `index/`. It is self-contained and relocatable; copying the directory is a complete deployment of the archive. It is data, not code: it lives outside Git and outside the backend binary, so neither has to be rebuilt or reshipped when the other changes.

Each index entry is the preferred copy of one article. It carries the identifier, title, and body (indexed with positions and stored), the publication date as a day number, the language, every source that published it, and its category; date, language, source, and category are fast fields, so filters, sorting, facets, and histograms read columns without loading documents. Stored only, for the reader: the date precision, the source address, the source's record identifier, and the source metadata. The translation group identifier is indexed so the other language editions are one term lookup. A copy held by a second source has no entry of its own: the preferred entry lists its source, stores its address, and indexes its identifier as an alias, so a link to the copy opens the preferred document.

Titles and bodies are most of the index. The document store is compressed with Zstandard at level 22 in 64 KiB blocks. Decompression speed does not depend on the level, so reading a block back costs microseconds while the bytes held in memory shrink; the index for 399,902 articles is 655 MB with this store and was 800 MB with LZ4 before the reader fields were added.

`kcna-archive database snapshot` brings the snapshot up to date and `database import` does the same after it stores documents. Each commit records the newest `document_updated_at` it has seen, so the next run reads only rows changed since. A changed row can alter an entry other than its own, because the preferred document it is, or was, a copy of lists it; those entries are rewritten in the same run. When the index holds more entries than the database has preferred documents, the stale ones are removed. An index that is missing, was written with another layout or other storage settings, or lacks the recorded timestamp is rebuilt in full; `--rebuild` forces that. The manifest is rewritten on every run with the snapshot format version, the build time, and the counts only the database knows: stored documents, folded copies, translation groups, and the last retrieval time.

## Memory

`Archive::load_into_memory` validates the manifest, maps each index file, and copies it once into an in-memory directory, so the only copy made is page cache to heap; reading a file into a buffer first would hold the largest file twice. The load refuses a directory with more than 4,096 files or 8 GiB, which stops a wrong path from exhausting the host. After the load the directory can be replaced or removed; requests read memory only. A server picks up a newer snapshot by loading again, which the backend exposes as a reload that swaps the new copy in while the old one keeps serving.

Measured on 2026-11-21 in a development build with optimized search dependencies, on the full corpus: a rebuild from PostgreSQL in 41 s using eight writer threads, a load into memory in 0.05 s with the files in the page cache, 731 MB resident afterwards, 1 to 16 ms for a result page with facets, 0.02 to 5.5 ms for a month histogram, and under 0.4 ms to open a document. `kcna-archive verify` repeats these measurements against any snapshot without a database.

## Search

Whitespace tokenization cannot search this corpus: Korean attaches particles to stems, and Chinese and Japanese have no word spaces. `search_text` is the single analyzer for indexing, querying, and highlighting. A run of non-CJK letters or digits is one lowercased word. A run of CJK characters becomes overlapping bigrams followed by its last character alone, so every CJK character starts exactly one lexeme. A query run of three or more CJK characters is a phrase of consecutive bigrams, which matches any substring; two characters are one bigram; one character is a prefix match over lexemes. Clauses are combined conjunctively and words in other scripts match whole.

Each clause may match in the title or the body, with the title weighted three times, and results sort by date, newest first unless `sort=oldest` or `sort=relevance` is given. Relevance ranks by BM25; without query text every score is equal, so it also means newest first. The analyzer also reports the byte range each lexeme came from; `search/highlight.rs` uses those ranges to return titles, snippets, and paragraphs as text segments with a hit flag, so the client never parses markup. A snippet is about 170 characters around the first match.

Facet counts are cross-filtered: the count beside a language ignores the language filter itself and respects every other filter, so the numbers show what selecting that value would return. Year and month histograms ignore the date range for the same reason, which lets a selected span be widened from the same chart; day buckets apply the range and are limited to 366 days.

Paging is offset-based and bounded: a page holds at most 100 results and a request may not reach past result 10,000. The collector is additionally capped by the corpus size because it reserves room for the requested window up front. An export holds at most 10,000 rows, and CSV fields that a spreadsheet would evaluate as a formula are prefixed with an apostrophe.

## What the snapshot leaves out

Raw response bytes stay in the collector's database. The reader links each article at its source address instead of a stored page. Serving stored pages would add the 2.7 GB raw corpus to the deployment for a view that is only needed when checking the importer.

## Deployment

`cargo xtask kcna-snapshot` writes the snapshot to `rust-be-template/data/kcna_archive`, where the development backend finds it; it runs the collector in the development profile and needs `DATABASE_URL` for the corpus database. `cargo xtask kcna-snapshot-push host:dist/data/kcna_archive/` copies it with rsync: changed files are staged and moved into place together at the end of the transfer, and because index segments are immutable an incremental snapshot transfers only its new segments. The backend reads `KCNA_ARCHIVE_DIR`, by default `data/kcna_archive` relative to its working directory, at startup; a running server keeps its loaded copy until it restarts or an administrator reloads the snapshot from the status page. A machine without the directory runs normally and reports the archive as not installed.
