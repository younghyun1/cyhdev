# cyhdev optimized binary measurements

All three requested variants passed complete functional coverage. Corrected native HTTPS measurements on September 30, 2026 found 272,837/308,685/306,277 small-health requests/s and 44,874/52,925/53,889 mixed requests/s for baseline/PGO/PGO+BOLT. PGO improved mixed throughput by 17.9%; PGO+BOLT by 20.1%. The original Playwright measurements were client-limited and cannot establish backend capacity or regressions. Their failed receipt remains unchanged; native remeasurement does not automatically export an accepted `dist/` artifact.

Status: correction complete on September 30, 2026. Checkout: `main`; implementation commits `c7a14d4` and `69c9c08`. The retained binaries still come from `1e88078`; no optimized recompilation was needed. The native runner, source checks, raw reports and CPU evidence replace Playwright's timed request loop. Verification: 14 workload tests, TypeScript, lint, Clippy for both changed Rust tooling packages, 61 xtask tests, two fixture tests and the two client-only reproduction cases passed. All 27,000,000 primary native requests succeeded. No remaining benchmark work; owned fixtures are stopped. Installation and production-shaped validation remain separate operations.

The original client progressively slowed because unscoped Playwright API calls search all prior test steps to locate their parent. A client-only reproduction against a trivial HTTP server fell from 2,684 to 1,314 requests/s across nine samples; an explicit enclosing `test.step` held rates near 2,560 requests/s. The compatibility benchmark now supplies that parent. Native `oha` mode moves measurement out of Playwright entirely while retaining semantic preflight and lifecycle checks.

## Artifacts and provenance

Files are retained under `target/optimization/cyhdev-pgo-bolt-003/`. [artifact-manifest.json](../../target/optimization/cyhdev-pgo-bolt-003/artifact-manifest.json) records backend/seed hashes, coverage/measurement hashes, merged-profile hashes, licenses and the parent receipt. Each variant uses its matching seed companion; PGO+BOLT uses the PGO companion.

| Variant | Backend file | SHA-256 |
| --- | --- | --- |
| Baseline | `baseline-rust-be-template` | `e8f200797cab2a7c1c75b170e1872f2fe7afa20d4b5a22d1c36856779e15c42f` |
| PGO | `pgo-rust-be-template` | `2cd9162c1c8186aef55782ebc39324f141fe79d2d260950b79319c64cf7ab3da` |
| PGO+BOLT | `bolt-rust-be-template` | `5af45439150589f5a93790b75a374a6e4a5a3d40742781ea9b2228d3c67979c6` |

Source: `1e8807893a2560b7958794fd25199e13ae412501`. Builder: `sha256:0ad1795afa51cff6ae573f26953e3b344e009570234667746f988229be64d6ec`. Compiler: Rust 1.100.0 nightly, commit `2e2b193f8`, LLVM 23.1.1; PGO merge used its bundled tools. BOLT: Debian 19.1.7, canonical executable and checked/hashed runtime archive. Runner: native Linux x86-64, AMD Ryzen 7 9800X3D; target CPU `znver3`. Variants share LTO, frame pointers, text relocations, rebuilt standard library, panic settings and embedded frontend/EU5. Baseline/PGO are unstripped BOLT inputs; the final BOLT file is stripped, so raw sizes are not a fair size-reduction comparison.

Campaign SHA-256: `1096ad73b7813e7b9f927b907db0bef3e257ba405026aff7aa7fdc32b71b61ec`. Golden fixture snapshot: `74bef75b798f3efc641e0bcc36f9db7c63676ebe1f63fe62a604a63f4b905a1a`. The 315 immutable fixture inputs hash to `252de4b777c30ee1d36e13ff46d27d0d187c390cbe2595f3c01c03f0c581637c`. Every candidate and benchmark reset to that snapshot. PostgreSQL, SMTP, S3, OIDC, worlds, control sockets and participants were disposable and synthetic.

## Corrected native measurements

Oha 1.16.0 used pooled HTTP/1.1 HTTPS, identity encoding, full response reads, 32 connections, four client threads and two-second request deadlines. Server, providers and disposable PostgreSQL were pinned to CPU 0-3; the driver/client were pinned to CPU 4-7. Backend Tokio had four workers. These are four-server-core results, not an eight-core extrapolation. Fresh snapshot clones supplied each backend start. Playwright checked each route's semantic response outside timing and released its connections before load. Native result validation requires every scheduled request to return HTTP 200 without transport errors; binary hashes were checked before and after each run.

Three rounds rotated baseline/PGO/BOLT, PGO/BOLT/baseline, and BOLT/baseline/PGO. Each process measured five samples after 10,000 warmup requests. Each health sample made 500,000 requests; each mixed sample made 100,000 requests. The health workload uses `/api/healthcheck/server`; the mixed workload uses the same ten public-read routes as the historical campaign. URL-file entries have equal selection probability. Each variant has 15 samples and 7,500,000 health plus 1,500,000 mixed requests. Every workload's compared reports match environment/workload identities; all requests succeeded. Values below are medians across samples, including initial connection setup amortized over subsequent reused connections.

| Workload | Variant | Requests/s | p95, ms | p99, ms |
| --- | --- | ---: | ---: | ---: |
| Small health JSON | Baseline | 272,837.41 | 0.177 | 0.239 |
| Small health JSON | PGO | 308,685.26 | 0.162 | 0.252 |
| Small health JSON | PGO+BOLT | 306,276.64 | 0.162 | 0.254 |
| Ten-route public reads | Baseline | 44,874.12 | 2.380 | 2.967 |
| Ten-route public reads | PGO | 52,925.46 | 2.020 | 2.524 |
| Ten-route public reads | PGO+BOLT | 53,888.61 | 1.987 | 2.483 |

PGO increased health throughput by 13.1% and mixed throughput by 17.9%. PGO+BOLT increased them by 12.3% and 20.1%. BOLT's incremental mixed gain over PGO was 1.8%; its health throughput was 0.8% lower than PGO. Both optimized variants improved mixed throughput in every rotated round. No samples or outliers were discarded.

All mixed comparisons meet the unchanged 98% throughput and 105% latency budgets. Health p99 was 5.4% higher with PGO and 6.3% higher with BOLT, despite higher throughput and lower p95, so those diagnostic tail comparisons exceed the same 5% budget. These saturated closed-loop percentiles describe the selected concurrency; they do not establish latency at a fixed arrival rate. The historical failure remains in its original receipt; the new evidence is recorded independently as measured, not promoted.

[native-correction/summary.json](../../target/optimization/cyhdev-pgo-bolt-003/native-correction/summary.json) seals the binary/source identities, comparison reports, native raw/CPU hashes and aggregate metrics. [comparison-records.json](../../target/optimization/cyhdev-pgo-bolt-003/native-correction/comparison-records.json) retains round order and per-process results. Each stage directory contains its exact campaign, private driver log, report and `native-loadgen/` raw evidence. Backend source is unchanged; harness source is `69c9c089625dd4129ff5a605daa8c0ea5d15015d`.

## Client headroom and concurrency

The baseline sweep found a health plateau at 16-32 connections; 64 connections mostly raised latency. Mixed throughput similarly stopped improving around 32 connections. Sweep samples were shorter and serve only to choose the repeated comparison's concurrency.

| Connections | Health requests/s | Mixed requests/s |
| ---: | ---: | ---: |
| 1 | 48,434 | 9,507 |
| 4 | 156,064 | Not sampled |
| 8 | 233,944 | 37,230 |
| 16 | 265,189 | Not sampled |
| 32 | 267,566 | 41,536 |
| 64 | 263,216 | 41,397 |

At the fixed server allocation and 32 connections, one/four/eight native client threads achieved 203,404/272,837/273,978 health requests/s and 44,842/44,874/45,140 mixed requests/s. One thread limited the small-response workload; doubling four to eight did not materially raise either rate. During the primary comparisons, the client used medians of 1.46-1.60 CPU cores for health and 0.41-0.48 for mixed traffic out of its four assigned cores. Backend process usage was 3.33-3.46 and 2.38-2.58 cores respectively; this excludes PostgreSQL/provider CPU on the server allocation. Peak observed client RSS stayed below 240 MiB. These checks support adequate native-client headroom for the recorded comparison. [sweep-summary.json](../../target/optimization/cyhdev-pgo-bolt-003/native-correction/sweep-summary.json) retains the individual points; the native summary retains thread calibration and CPU evidence.

## Historical client-limited measurements

Nine samples per variant, 3,000 successful public-read HTTPS requests per sample, 500 warmup requests and concurrency eight. Ten equally weighted routes use reused loopback connections. Values are medians across samples; p95/p99 include client transport and semantic checks. Every sample had zero failures.

| Variant | Requests/s | p95, ms | p99, ms |
| --- | ---: | ---: | ---: |
| Baseline | 1,050.78 | 10.61 | 11.43 |
| PGO | 1,033.71 | 10.81 | 12.27 |
| PGO+BOLT | 1,050.47 | 10.66 | 11.69 |

The unchanged budgets require throughput at least 98% of the reference and p95/p99 at most 105%. PGO throughput was 98.38% of baseline, p95 101.90%, and p99 107.34%; p99 caused rejection. BOLT was within budgets relative to baseline and PGO, but cannot bypass the PGO gate. [receipt.json](../../target/optimization/cyhdev-pgo-bolt-003/receipt.json) records `failed`.

## Historical client-limited rotated repeats

Three rounds rotated execution order: baseline/PGO/BOLT, PGO/BOLT/baseline, BOLT/baseline/PGO. Each process measured five samples of 3,000 requests after 500 warmup requests; each variant therefore has 15 samples. Source, binary hashes, fixture snapshot, environment, request weights, concurrency and topology matched. The shorter per-process sample count changes campaign identity; compare variants within this pass rather than mixing its absolute rates with the primary pass.

| Variant | Requests/s | p95, ms | p99, ms |
| --- | ---: | ---: | ---: |
| Baseline | 1,214.91 | 9.22 | 9.98 |
| PGO | 1,213.84 | 9.20 | 10.00 |
| PGO+BOLT | 1,226.62 | 9.09 | 9.86 |

All samples had zero failures and all diagnostic comparisons met the original budgets. The client-limited rates differed by -0.09% for PGO and +0.96% for BOLT. The original tail regression did not reproduce in these shorter passes. The later client-only reproduction established that test-history traversal biased both passes; their earlier inference that optimization gains were negligible is superseded by the native comparison. [repeatability/summary.json](../../target/optimization/cyhdev-pgo-bolt-003/repeatability/summary.json) retains the historical report hashes, identities, order and comparisons. It does not replace or promote the rejected original receipt.

## Coverage and operational outcome

Baseline, PGO instrumentation, PGO, BOLT instrumentation and final stripped BOLT each passed 112 HTTP operations, 33 routes across 264 UI states, and 31 scenarios. Coverage includes real capability mail, account lifecycle/OIDC/authority changes, nested content/moderation, image/WASM processing, retention jobs, host/chat WebSockets, bidirectional fake-device RTC audio/video, all Minecraft dimensions/tile transports, permission expiry, surveys/blocks/waypoints, fixture control acknowledgements, navigation guards, compression and embedded EU5 rendering/search. The normal prediction launcher flushed its native default profile; PGO merged two fresh backend/worker profiles. BOLT trained the exact PGO ELF, merged fresh profiles, optimized layout and passed final PIE/provenance checks. Some AWS-LC assembly functions were left unchanged by BOLT's patchability checks.

Fixtures use small synthetic media and loopback providers; functional coverage does not certify production provider behavior or production capacity. Owned fixture processes were stopped after all measurements. Binaries, profiles, licenses, immutable builder image, private logs and receipts remain local. No production service, configuration, installation or deployment was changed.
