# Performance and security review

Review date: September 21, 2026. Scope: local source review, dependency/secret checks, browser performance, and the changed visitor-map and live-chat boundaries. This is not a penetration test, production traffic measurement, or deployment approval. Existing external credential-rotation and dependency-promotion obligations in `TODO` remain open.

## Changes

Visitor-map translations now enter Leaflet popups and failure UI as text rather than HTML. A disposed-page guard prevents a late request from creating a map after navigation. Live-chat deletion checks database-current superuser authority under row locks, erases message content, invalidates bounded cache indexes after commit, and broadcasts invalidation without logging message text. The HTTP task completes post-commit invalidation after client disconnection. Client message history and recent deletion IDs are bounded to 300 each.

## Performance evidence

`npm --prefix solid-csr-spa-template run test:e2e:performance` passed the existing five-run cold-cache mobile median thresholds, with Chromium CPU/network throttling. Results describe deterministic fixtures on this machine, not live backend response times.

| Route | Median LCP | Median measured interaction | Median CLS |
| --- | --- | --- | --- |
| `/` | 1124 ms | 24 ms | 0.000089 |
| `/blog` | 1376 ms | 32 ms | 0.000045 |
| `/forum` | 1336 ms | 32 ms | 0.026723 |
| `/photographs` | 1404 ms | 32 ms | 0.075807 |
| `/login` | 1308 ms | 32 ms | 0.006070 |
| `/live-chat` | 1360 ms | 40 ms | 0.000770 |

The frontend build measured 124.3 KiB gzip for the initial module graph across five assets and passed its budget. Route-heavy editor, chart, and Leaflet bundles remain lazy. Native throughput and release-image benchmarks were deliberately not run because they require prohibited release builds. The photograph CLS is below the gate but is the largest measured shift; verify real image dimensions and loading placeholders before further polish.

## Dependency findings

`npm audit --json` and its production-only variant reported zero vulnerabilities across the locked frontend dependency graph. `cargo audit --json` is not clean:

- `rsa 0.9.10`, through `openidconnect 4.0.1`: [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), timing leakage from private-key operations, has no patched version in the consulted advisory. Local OIDC code performs provider-token verification and uses client-secret authentication; no RSA private-key signing/decryption path was found. This is a source-based exposure assessment, not removal of the vulnerable dependency.
- `lru 0.16.4`, through `tantivy 0.26.2`: [RUSTSEC-2026-0253](https://rustsec.org/advisories/RUSTSEC-2026-0253.html), panic-safety unsoundness in `pop()`, is fixed in `>=0.18.2`. The inspected Tantivy cache uses `usize` keys, which do not have the potentially panicking destructor required by this advisory's scenario. The old dependency still needs an upstream-compatible upgrade; no unsupported transitive-major replacement was made.
- Removed the `bincode 1.3.3` and `yaml-rust 0.4.5` warnings by disabling Comrak's unused default CLI and syntax-highlighting dependencies. The application calls the library Markdown renderer without a syntax-highlighting adapter. Explicit emoji/shortcode support remains enabled; Markdown compatibility and unsafe-HTML regression tests cover the reduced feature set.
- The unmaintained `paste 1.0.15` warning remains through `image`'s `ravif`/`rav1e` AVIF encoder and `exr`/`pulp` decoder. AVIF is the application's output encoding, so disabling that chain would remove required behavior. [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) identifies a maintenance concern rather than a disclosed exploit. Replacing transitive packages with a fork needs a separate compatibility decision.

Primary advisories were consulted on the review date. No versions or advisories were suppressed to produce a passing audit.

## Confidential-information checks

`cargo xtask secret-scan` now passes after explicit recursive handling of indexed Git links. The scanner snapshots public source from the root and initialized submodules, then scans all refs in each repository independently. All three current reports are empty: the combined public-source snapshot, root history, and EU5 history. This corrects the earlier unsupported-directory failure; that earlier failed gate did not establish source confidentiality.

Missing/uninitialized submodules fail with an initialization command rather than being skipped. Unexpected directories, symlinked ancestors/submodules, unresolved Git indexes, and tracked credential-shaped files still fail closed. Ignored runtime credentials are classified by filename and metadata without reading their contents. Traversal has repository/depth limits, and recursive source shares aggregate file/byte limits. See the [scanner/dependency follow-up](2026-09-21-secret-scanner-dependencies.md) for verification. Production credential rotation and revocation require separate operational evidence.
