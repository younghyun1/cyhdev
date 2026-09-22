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
- Unmaintained dependency warnings remain for `bincode 1.3.3`, `paste 1.0.15`, and `yaml-rust 0.4.5`. Resolve these through their owning dependencies in a dedicated upgrade pass, with compatibility tests.

Primary advisories were consulted on the review date. No versions or advisories were suppressed to produce a passing audit.

## Confidential-information checks

`cargo xtask secret-scan` failed before scanning because its source inventory treats the `vendor/eu5-location-filter` Git submodule directory as an unsupported file type. This is a scanner limitation, not a finding of a leaked secret, and the aggregate gate must not be reported as passing. The fallback `gitleaks git --redact --no-banner --log-opts=--all` scanned 980 commits through implementation commit `e449780` with no findings. Repeat it after committing this report before pushing. It does not inspect ignored local runtime files or independently establish the confidentiality of a submodule's own history.

Follow-up: teach the source scanner to represent Git links explicitly while preserving fail-closed handling of unexpected directories, and test initialized/uninitialized submodules. Preserve its refusal to read private runtime credentials. Production credential rotation and revocation require separate operational evidence.
