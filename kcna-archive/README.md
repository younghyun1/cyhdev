# KCNA research archive

This part of the workspace collects public Korean Central News Agency material for research and serves it on the website to administrator accounts. `cli/` is the collector, importer, and snapshot writer (`kcna-archive`); `core/` is the search index, snapshot format, and read-only service (`kcna-archive-core`) that the backend links. The browser lives in the site's frontend under `solid-csr-spa-template/src/pages/kcna_archive/`.

It collects public Korean Central News Agency material for local research. Korean text is the first priority; KCNA-published English, Chinese, Russian, Spanish, and Japanese versions are retained as parallel source records. Downloads are resumable and content-addressed in an append-only retrieval manifest.

## Sources

| Source | Coverage | Access used here | Notes |
| --- | --- | --- | --- |
| [KCNA](http://www.kcna.kp/kp) | Current Korean archive plus five translation languages | Direct public pages | The site declares `robots=ALL`; archive pagination uses CSRF-protected POST forms, which the collector enumerates. |
| [Korea News Service](http://www.kcna.co.jp/) | Korean from 1998, English from 1996, Spanish from 2002; currently indexed through 2025 | Direct public static pages | This is the historical Japan-hosted KCNA distribution site. Pages retain their original EUC-KR, Shift-JIS, or UTF-16 bytes. |
| [Korea News Service current site](https://koreanewsservice.com/) | Korean and English from late 2025 onward | Not downloaded | The site is public and carries KCNA-supplied text, but its site policy expressly prohibits copying articles into databases. Current material is collected from KCNA's own site instead. |
| [Open Korean Historical Corpus](https://huggingface.co/datasets/seyoungsong/Open-Korean-Historical-Corpus) | 170,472 Korean, 153,433 English, and 30,042 Spanish metadata records dated 1997-2025 | Public URL inventory | Article text is redacted because the source is all rights reserved. The verified 223,707,089-byte file has SHA-256 `334b0f8b40d63aac5a8387065a076a922e4a9d5cdc21d09c18b2142dc2862532`; it supplies exact historical seed URLs. |
| [Internet Archive](https://web.archive.org/web/*/kcna.kp) | Partial snapshots from 2011 onward; useful coverage resumes in 2022 | Gap analysis and fallback only | KCNA used JavaScript-only article loading between 2011 and 2022, so Wayback did not preserve a complete corpus during that interval. |
| [North Korea state news agency archive](https://github.com/stiles/north-korea-news) | 14,472 English records dated December 2023-November 2026; 11,992 include article text | Public CC0 JSON snapshot | This daily collector is a secondary cross-check. The snapshot retrieved on 2026-11-21 has SHA-256 `d49056eedcc73704885ec0de4fdf51419363a4c245243098926fc07c4e39932d`; the 2026-08-30 snapshot is kept beside it as `headlines.2026-08-30.json`. |
| [Harvard Dataverse Korean corpus](https://doi.org/10.7910/DVN/Z80EZZ) | 105,000 Korean articles, October 2013-January 2021 | Not downloaded | The sole spreadsheet is restricted and requires an access request. |
| [FOCUSdata](https://dataverse.harvard.edu/dataverse/focusdataproject) | About 85,000 English articles, October 2008-February 2020 | Not downloaded | Access is request-based and the collection is English-only. |
| [KCNA Watch](https://kcnawatch.org/) | Broad commercial archive from 1997 | Not downloaded | It is useful for manual gap checks, but its proprietary database is outside this collector's scope. |

All KCNA and Korea News Service article text remains subject to the source publisher's terms. The local copy is intended for research preservation, not redistribution. Dataset licensing does not supersede article copyright.

## Run

The project uses rolling async collection with bounded concurrency, per-source request pacing, six retry attempts with jittered exponential backoff, a 32 MiB response cap, atomic file replacement, and a fixed 500,000-URL crawl ceiling. Existing raw pages are reused on the next run.

The collector builds with the workspace toolchain and profiles. Run it from the repository root; `--data-dir` names the corpus directory, which is `data` relative to the working directory unless given. The examples below assume the corpus is in `kcna-archive/data`.

```bash
cargo run --package kcna-archive -- --data-dir kcna-archive/data all \
  --seed-index kcna-archive/data/indexes/okhc/kcna_jp.jsonl \
  --live-concurrency 12 \
  --legacy-concurrency 32
```

Download and verify the public URL inventory first:

```bash
mkdir -p kcna-archive/data/indexes/okhc
curl -fL --retry 6 --retry-all-errors --continue-at - \
  --output kcna-archive/data/indexes/okhc/kcna_jp.jsonl \
  'https://huggingface.co/datasets/seyoungsong/Open-Korean-Historical-Corpus/resolve/main/kcna_jp.jsonl?download=true'
openssl dgst -sha256 kcna-archive/data/indexes/okhc/kcna_jp.jsonl
```

The main subcommands are `all`, `live`, and `legacy`. `--languages kp,en` limits live collection while preserving the stated priority order; an unknown language is rejected. `legacy` without `--seed-index` crawls outward from the site's index pages and also finds pages newer than the inventory. A completed run exits nonzero if any page remains unavailable after retries; rerunning the same command retries only missing files. Pages the host reports as nonexistent (HTTP 404 or 410) are counted as `missing` and do not fail the run: the mirror's own index pages link to about 20,000 pages it does not serve. They are remembered in `manifests/dead-links.jsonl` and not requested again for 30 days; `--recheck-missing` overrides that. In `all`, the two sources run independently, so an unreachable host does not cancel the other.

To pick up newly published material, rerun `live`. KCNA keeps a rolling window of articles and its list pagination shifts with every publication, so each run fetches every list page again and downloads the articles that are not stored yet. An interrupt (Ctrl-C or SIGTERM) stops the run after flushing the retrieval manifest.

## Layout

```text
data/
  indexes/okhc/kcna_jp.jsonl     public historical URL inventory
  indexes/stiles/headlines.json  public recent English cross-check
  snapshot/                      what the website serves: index/ and snapshot.json
  manifests/retrievals.jsonl     retrieval time, URL, corpus path, size, SHA-256
  manifests/dead-links.jsonl     pages the historical mirror reported as nonexistent
  raw/kcna_co_jp/                 historical raw source bytes by URL path
  raw/kcna_kp/lists/              live archive pagination evidence
  raw/kcna_kp/articles/           live articles by language and source ID
```

Manifest paths are relative to the data directory (`raw/...`), so the corpus can be moved or addressed through a different `--data-dir`; entries written by earlier versions as `data/raw/...` resolve the same way. List pages are the only source of live publication dates. The first snapshot of a page is `{page}.html`; a later run adds `{page}.r{unix-time}.html` only when that page names an article not yet stored, and never overwrites an earlier snapshot.

The `data` tree is intentionally excluded from Git because the corpus is large and the underlying text is not released under this repository's software license.

## PostgreSQL 18

The embedded schema stores normalized full text, complete source metadata JSON, immutable raw response bytes, hashes, charset and media type, and retrieval provenance. Translation groups and categories are tables of their own, and copies of one article in several sources point at a preferred copy. Multi-article historical pages store one raw snapshot linked to each extracted document. The importer uses `diesel_async`, a bounded bb8 pool, concurrent source pipelines, page parsing on the blocking thread pool, and 250-row transactions by default.

```bash
export DATABASE_URL='postgres://user:password@localhost/kcna'
cargo run --package kcna-archive -- --data-dir kcna-archive/data database migrate
cargo run --package kcna-archive -- --data-dir kcna-archive/data database import
cargo run --package kcna-archive -- --data-dir kcna-archive/data database snapshot
```

The import is idempotent and can be rerun after every collection; documents that have not changed are skipped. It then links copies across sources and updates the snapshot in `<data-dir>/snapshot` (or `--snapshot-dir`), so `database snapshot` is needed only on its own after restoring a database or with `--rebuild` to discard the index. Besides the records listed in the inventory, the import covers stored `kcna.co.jp` article pages that the inventory does not name (June to October 2025), with the date and language taken from the file name and `publication_precision = inferred_day`. After upgrading from a version before 2026-11-21, run `database migrate` and `database import` once: the migrations move translation groups and categories into their own tables and drop the PostgreSQL search column.

PostgreSQL 18 mints document UUIDv7 values with their publication timestamp. Most KCNA records expose only a date; those use noon in `Asia/Pyongyang` as an ordering anchor and retain `publication_precision = day` plus the original date string. A database check verifies that each UUIDv7 timestamp matches its source timestamp within 25 milliseconds, so a document whose date is corrected is replaced under a new identifier. Retrieval UUIDs similarly encode the actual retrieval time. See [the storage design](../docs/architecture/be/kcna-archive-corpus.md) for provenance and precision details.

Unit tests need no services. `KCNA_TEST_DATABASE_URL` additionally enables a round-trip test against a disposable PostgreSQL 18 database:

```bash
KCNA_TEST_DATABASE_URL='postgres://user:password@localhost/kcna_test' cargo test --package kcna-archive
```

## Website

The site serves the archive at `/projects/kcna-archive` to superuser accounts only, because the API would otherwise be open and the corpus is not redistributable. It reads a snapshot into memory at startup and needs no database for the archive. From the repository root:

```bash
export DATABASE_URL='postgres://user:password@localhost/kcna'
cargo xtask kcna-snapshot                 # write rust-be-template/data/kcna_archive from the corpus database
cargo run --package kcna-archive -- --snapshot-dir rust-be-template/data/kcna_archive verify
cargo xtask backend                       # the development backend loads the snapshot at startup
cargo xtask kcna-snapshot-push host:dist/data/kcna_archive/
```

`verify` loads the snapshot the way the server does and logs its size, load time, and the timing of a fixed set of requests; it needs no database. After a push, a running server keeps its loaded copy until it restarts or an administrator presses Reload snapshot on the status page. A full rebuild of the 400,000-article corpus takes under a minute and an unchanged corpus a fraction of a second, so the snapshot can be refreshed after every collection.

The section is one search over the whole archive. A query combines text, languages, sources, a date range, a category, and a sort order, and lives in the URL, so every view is a link. Without text it browses. A histogram shows matching documents per month and narrows the date range when dragged across. Results carry highlighted snippets and name every source that holds the article; the reader opens another language edition side by side and links the article at its source. Trends compares up to four terms over time and Status summarizes the corpus and the loaded snapshot. Result sets export as CSV or JSON Lines.

Search is substring-accurate for Korean, Chinese, and Japanese: `김정은` finds `김정은동지께서`, and a single character such as `핵` finds every word containing it. Words in other scripts match whole and case-insensitively. See [archive serving](../docs/architecture/be/kcna-archive-serving.md), [the backend feature](../docs/architecture/be/kcna-archive-feature.md), and [the browser design](../docs/design/fe/kcna-archive-browser.md).
