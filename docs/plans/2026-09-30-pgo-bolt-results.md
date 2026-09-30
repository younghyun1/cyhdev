# cyhdev optimized binary measurements

All three requested variants were built and benchmarked on September 30, 2026: optimized baseline, PGO, and PGO+BOLT. All passed complete functional coverage. The primary campaign failed performance acceptance because PGO p99 exceeded baseline by 7.3%; its failed receipt remains unchanged and no accepted `dist/` artifact was exported.

## Artifacts and provenance

Files are retained under `target/optimization/cyhdev-pgo-bolt-003/`. [artifact-manifest.json](../../target/optimization/cyhdev-pgo-bolt-003/artifact-manifest.json) records backend/seed hashes, coverage/measurement hashes, merged-profile hashes, licenses and the parent receipt. Each variant uses its matching seed companion; PGO+BOLT uses the PGO companion.

| Variant | Backend file | SHA-256 |
| --- | --- | --- |
| Baseline | `baseline-rust-be-template` | `e8f200797cab2a7c1c75b170e1872f2fe7afa20d4b5a22d1c36856779e15c42f` |
| PGO | `pgo-rust-be-template` | `2cd9162c1c8186aef55782ebc39324f141fe79d2d260950b79319c64cf7ab3da` |
| PGO+BOLT | `bolt-rust-be-template` | `5af45439150589f5a93790b75a374a6e4a5a3d40742781ea9b2228d3c67979c6` |

Source: `1e8807893a2560b7958794fd25199e13ae412501`. Builder: `sha256:0ad1795afa51cff6ae573f26953e3b344e009570234667746f988229be64d6ec`. Compiler: Rust 1.100.0 nightly, commit `2e2b193f8`, LLVM 23.1.1; PGO merge used its bundled tools. BOLT: Debian 19.1.7, canonical executable and checked/hashed runtime archive. Runner: native Linux x86-64, AMD Ryzen 7 9800X3D; target CPU `znver3`. Variants share LTO, frame pointers, text relocations, rebuilt standard library, panic settings and embedded frontend/EU5. Baseline/PGO are unstripped BOLT inputs; the final BOLT file is stripped, so raw sizes are not a fair size-reduction comparison.

Campaign SHA-256: `1096ad73b7813e7b9f927b907db0bef3e257ba405026aff7aa7fdc32b71b61ec`. Golden fixture snapshot: `74bef75b798f3efc641e0bcc36f9db7c63676ebe1f63fe62a604a63f4b905a1a`. The 315 immutable fixture inputs hash to `252de4b777c30ee1d36e13ff46d27d0d187c390cbe2595f3c01c03f0c581637c`. Every candidate and benchmark reset to that snapshot. PostgreSQL, SMTP, S3, OIDC, worlds, control sockets and participants were disposable and synthetic.

## Primary measurements

Nine samples per variant, 3,000 successful public-read HTTPS requests per sample, 500 warmup requests and concurrency eight. Ten equally weighted routes use reused loopback connections. Values are medians across samples; p95/p99 include client transport and semantic checks. Every sample had zero failures.

| Variant | Requests/s | p95, ms | p99, ms |
| --- | ---: | ---: | ---: |
| Baseline | 1,050.78 | 10.61 | 11.43 |
| PGO | 1,033.71 | 10.81 | 12.27 |
| PGO+BOLT | 1,050.47 | 10.66 | 11.69 |

The unchanged budgets require throughput at least 98% of the reference and p95/p99 at most 105%. PGO throughput was 98.38% of baseline, p95 101.90%, and p99 107.34%; p99 caused rejection. BOLT was within budgets relative to baseline and PGO, but cannot bypass the PGO gate. [receipt.json](../../target/optimization/cyhdev-pgo-bolt-003/receipt.json) records `failed`.

## Rotated diagnostic repeats

Three rounds rotated execution order: baseline/PGO/BOLT, PGO/BOLT/baseline, BOLT/baseline/PGO. Each process measured five samples of 3,000 requests after 500 warmup requests; each variant therefore has 15 samples. Source, binary hashes, fixture snapshot, environment, request weights, concurrency and topology matched. The shorter per-process sample count changes campaign identity; compare variants within this pass rather than mixing its absolute rates with the primary pass.

| Variant | Requests/s | p95, ms | p99, ms |
| --- | ---: | ---: | ---: |
| Baseline | 1,214.91 | 9.22 | 9.98 |
| PGO | 1,213.84 | 9.20 | 10.00 |
| PGO+BOLT | 1,226.62 | 9.09 | 9.86 |

All samples had zero failures and all diagnostic comparisons met the original budgets. PGO throughput differed by -0.09%; BOLT by +0.96% against baseline. The original tail regression did not reproduce in the shorter rotated passes. The absolute shift between passes and small differences between variants limit interpretation; this is no calibrated capacity claim or statistically established speedup. [repeatability/summary.json](../../target/optimization/cyhdev-pgo-bolt-003/repeatability/summary.json) retains report hashes, identities, order and comparisons. It does not replace or promote the rejected primary result.

## Coverage and operational outcome

Baseline, PGO instrumentation, PGO, BOLT instrumentation and final stripped BOLT each passed 112 HTTP operations, 33 routes across 264 UI states, and 31 scenarios. Coverage includes real capability mail, account lifecycle/OIDC/authority changes, nested content/moderation, image/WASM processing, retention jobs, host/chat WebSockets, bidirectional fake-device RTC audio/video, all Minecraft dimensions/tile transports, permission expiry, surveys/blocks/waypoints, fixture control acknowledgements, navigation guards, compression and embedded EU5 rendering/search. The normal prediction launcher flushed its native default profile; PGO merged two fresh backend/worker profiles. BOLT trained the exact PGO ELF, merged fresh profiles, optimized layout and passed final PIE/provenance checks. Some AWS-LC assembly functions were left unchanged by BOLT's patchability checks.

Fixtures use small synthetic media and loopback providers; functional coverage does not certify production provider behavior or production capacity. Owned fixture processes were stopped after all measurements. Binaries, profiles, licenses, immutable builder image, private logs and receipts remain local. No production service, configuration, installation or deployment was changed.

