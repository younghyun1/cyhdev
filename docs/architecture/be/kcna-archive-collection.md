# Collection

Collectors write immutable raw bytes under `data/raw` and append one line per stored response to `data/manifests/retrievals.jsonl`. A file is written to a sibling `.part` name and renamed, so a reader never sees a partial page. Manifest paths are relative to the data directory. The manifest is flushed every 32 records and on every exit path, including errors and SIGINT or SIGTERM; a raw file without a manifest line still imports, with its modification time recorded as the retrieval time and flagged as recovered.

## Transport

Each source has one `Fetcher`: a shared connection pool, a semaphore bounding requests in flight, and a minimum interval between request starts. A request is attempted six times with exponential backoff and jitter. Transport failures, truncated bodies, and HTTP 408, 429, and 5xx are retried; other statuses, redirect loops, and unbuildable requests fail at once. Bodies are read incrementally and abandoned beyond 32 MiB.

## Live site (kcna.kp)

Each language edition is enumerated from its category list pages and then every linked article is fetched. Pagination is a CSRF-protected POST form. KCNA keeps a rolling window of articles and inserts new ones at the front, so a page number never describes the same articles twice. Every run therefore fetches every list page again; reusing stored pages would skip whatever was published between runs beyond the first page.

List pages are also the only source of publication dates, so the importer needs a stored page naming each article. The first snapshot of a page number is `{page}.html`. A later run stores `{page}.r{unix-time}.html` only when the page names an article that is not stored yet, and never replaces an earlier snapshot, because overwriting would drop the dates of articles that shifted to the next page. An article response is accepted only if it parses as an article; KCNA occasionally answers 200 with an empty shell, which is retried.

One failing category or edition is counted and logged without discarding the rest, and in `all` the live and historical collectors run to completion independently.

## Historical mirror (kcna.co.jp)

With a seed index the collector fetches exactly the listed URLs and confirms resumed pages by presence. Without one it crawls from the index pages, following `a[href]` and `frame[src]` links to `.htm` and `.html` pages on the same host, with a rolling window of pages in flight. The host is checked before a URL is canonicalized: canonicalization rewrites the host, so the opposite order would turn every external link into a request to the mirror.

The mirror's index pages link to about 20,000 pages it never served. A 404 or 410 response is appended to `data/manifests/dead-links.jsonl` with the time it was seen, and later runs skip those URLs for 30 days and count them as `missing` without a request; `--recheck-missing` asks for all of them again. The file is rewritten at the start of each run with expired and repeated entries dropped, so it is bounded by the number of dead links. On 2026-11-21 this cut a discovery run with nothing new to fetch from 7 m 22 s to 43 s.

## Decoding

Pages are stored as received and decoded only when read. The decoder is chosen by byte-order mark, then by the declared `charset` label resolved through the WHATWG label table (which covers `ks_c_5601-1987`, `x-sjis`, and maps `iso-8859-1` to windows-1252), then by content. About 1,800 Korean pages declare nothing: an undeclared page that is not valid UTF-8 is EUC-KR when at least 5% of its bytes are non-ASCII and it decodes with few errors, and windows-1252 otherwise, which covers English and Spanish pages with stray Latin-1 punctuation.
