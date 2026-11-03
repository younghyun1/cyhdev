# PGO and BOLT rotated benchmarks

Status: complete on November 3, 2026. The managed PGO and BOLT campaign now benchmarks all three binaries together in rotated rounds, and a re-benchmark confirmed that the rejected November 3 binaries were faster than baseline.

Checkout: `main`; implementation commit `6d41e8b` on top of `4359979`, which built the measured binaries. No uncommitted paths.

## Problem

The first managed run on November 3, `cyhdev-pgo-bolt-18e49bd91e020260_38bf3`, compiled and passed coverage for every stage, then failed acceptance with "pgo regresses beyond budgets relative to baseline". Each binary had been benchmarked once, right after its build: baseline at 11:41, PGO at 12:12 and BOLT at 12:25 local time. Medians were 55,542, 48,808 and 47,432 requests/s. Yet PGO used 54 µs of server CPU per request against baseline's 69 µs, and kept fewer cores busy, so the server waited more rather than worked more.

The fixture database is restored from one snapshot before every run, and no campaign container or other build overlapped a benchmark window. Desktop applications started at 12:19, and the host runs amd-pstate with the balanced profile (`powersave` governor, `balance_performance` preference, SMT off). Runs half an hour apart therefore measured host drift as much as the binaries, against 2 percent throughput and 5 percent latency budgets.

## Completed

- **Rotated rounds:** `tools/xtask/src/optimization/rounds.rs` benchmarks baseline, PGO and BOLT after all builds, in `benchmark_rounds` rounds (default 3, a multiple of three from 3 to 12) whose order rotates so each stage runs first, second and last once per three rounds. Reports are `{stage}-benchmark-round-N.json`; raw samples are under `native-loadgen/{stage}/round-N/`.
- **Paired comparison:** `measurement::ratios` takes one candidate-to-reference ratio of median throughput, p95 and p99 per round and accepts on the median ratio. Per-round medians and ratios are printed and recorded in `receipt.json`.
- **Host policy warning:** the campaign records the CPU frequency governor and energy preference and warns unless one of them is `performance`.
- **Re-benchmark mode:** `./build_pgo_and_bolt.sh --rebenchmark RUN_DIR` (`cargo xtask optimize rebenchmark CONFIG.json RUN_DIR` once fixtures exist) prepares fresh fixtures, reuses the earlier run's `surface.json` instead of the release-profile inventory export, copies that run's binaries, and compares them without compiling or exporting anything.

## Verification

`cargo xtask fmt`, `cargo xtask clippy`, `cargo test --locked -p xtask` (82 passed, including five new comparison tests) and `npm --prefix solid-csr-spa-template run optimization:check` (17 tests) passed. A guarded `cargo build --release -p optimization-fixtures` in the bootstrap's environment reported `Finished` with no unit recompiled before the live run.

`cargo xtask build-pgo-and-bolt --rebenchmark target/optimization/cyhdev-pgo-bolt-18e49bd91e020260_38bf3` ran as `cyhdev-pgo-bolt-18e4a222e2b1355f_4d1f4` in 210 s on the same balanced host policy and exited successfully with status `within_budget`:

| Round order | baseline | PGO | BOLT |
| --- | --- | --- | --- |
| 1: baseline, PGO, BOLT | 49,399 | 58,665 | 60,034 |
| 2: PGO, BOLT, baseline | 48,942 | 59,362 | 58,773 |
| 3: BOLT, baseline, PGO | 49,797 | 58,368 | 58,414 |

Median paired ratios: PGO against baseline 1.188x throughput, 0.836x p95, 0.821x p99; BOLT against baseline 1.201x, 0.823x, 0.828x; BOLT against PGO 1.001x, 1.005x, 1.016x. This agrees with the September 30 campaign in [the earlier results](2026-09-30-pgo-bolt-results.md), where PGO improved mixed throughput by 17.9 percent and PGO+BOLT by 20.1 percent. Fixtures were stopped and removed afterwards.

Not run: a full campaign with the new rotation, which requires optimized builds; and a run under the performance power profile.

## Remaining

None for this change. The November 3 BOLT binary has no `dist/` export because its original run failed acceptance; it is `target/optimization/cyhdev-pgo-bolt-18e49bd91e020260_38bf3/bolt-rust-be-template` with `pgo-minecraft-seed`. The next full `./build_pgo_and_bolt.sh` exercises rotation end to end; run `powerprofilesctl set performance` first.
