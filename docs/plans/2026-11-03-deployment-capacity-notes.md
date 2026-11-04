# Deployment capacity and compression notes

Status: open, not started; recorded November 3, 2026 for future optimization. No code was changed. These notes collect estimates and known gaps from the [rotated PGO and BOLT re-benchmark](2026-11-03-pgo-bolt-rotated-benchmarks.md) and the [September 30 results](2026-09-30-pgo-bolt-results.md).

Checkout: `main` at `dde6924`; no uncommitted paths.

## Measured inputs

Server-side CPU per request, PGO and BOLT build on the Ryzen 7 9800X3D workstation (SMT off), keep-alive HTTPS, identity encoding, fixture data: about 46 µs (four pinned cores, September 30) to 52 µs (unpinned, November 3) in the backend process for the ten-route public-read mix, and at most about 74 µs including PostgreSQL and the fixture providers that shared the four server cores. The small health route costs about 11-13 µs. Baseline costs about 69 µs in the backend process.

Deployment host (`miniserver`, read-only inspection): AMD Ryzen 9 6900HX, 8 cores and 16 threads, 4.94 GHz maximum, 16 MiB L3, balanced power policy (`powersave` governor, `balance_performance` preference), Ethernet `eno1` linked at 1 Gb/s, load average 0.34 at inspection. It also runs PostgreSQL and the Minecraft server. The home internet upload rate is unknown.

Live response sizes from one request per route on cyhdev.com, in bytes:

| Route | Identity | zstd |
| --- | ---: | ---: |
| `/api/blog/posts` | 926 | 579 |
| `/api/blog/search?q=rust` | 203 | 172 |
| `/api/forum/topics` | 755 | 489 |
| `/api/photographs/get` | 15,535 | 2,666 |
| `/api/dropdown/country` | 53,732 | 7,556 |
| `/api/dropdown/language` | 18,296 | 2,671 |
| `/api/i18n/ui-text?locale=en-US` | 37,051 | 12,570 |
| `/api/healthcheck/server` | 131 | 118 |
| `/api/visitor-board` | 11,524 | 4,970 |

The mix averages about 15 KB uncompressed and 3.5 KB with zstd, against 11.3 KB in the benchmark fixture. A first visit downloads about 115 KiB of zstd-compressed initial JS and CSS plus 1 KB of HTML.

## Estimated miniserver ceilings without rate limiting

The 6900HX is treated as about 5-6 workstation cores: about 25 percent lower IPC (Zen 3+ against Zen 5), 15-20 percent lower sustained all-core clocks in its slim case, a sixth of the L3, and SMT adding 20-25 percent. Half a core is reserved for Minecraft and the OS, and scaling from four cores to 16 threads is assumed 80 percent efficient. The link column assumes about 117 MB/s of TCP payload on the 1 Gb/s link and 0.5-1 KB of headers per response. These are estimates within about 30 percent, not measurements.

| Workload | CPU ceiling | Link ceiling | Likely limit |
| --- | --- | --- | --- |
| Public-read mix, identity | 55-70k req/s | 7-8k req/s | Link |
| Public-read mix, zstd as browsers request | 30-45k req/s | 25-28k req/s | Roughly balanced near 25k req/s |
| Health JSON | 300-370k req/s | about 130k req/s | Link |
| Repeat page views, assets cached, about four API calls | 8-10k/s | 6-7k/s | 5-7k/s |
| First visits | Far higher | 700-850/s | Link, before fonts and images |

The home upload rate likely sits below the link column and would then be the real ceiling. Production rate limiting lets one IP sustain about 16 requests/s after a 1,024-request burst; the table assumes it is off or traffic comes from many addresses.

## Issues for future optimization

- **Per-response compression cost:** `rust-be-template/src/routers/main_router/compression.rs` applies `tower-http` zstd and gzip at the default level to every eligible API response, with a fresh encoder per response. Compressing a 15 KB response is estimated at 30-40 µs, comparable to the roughly 50 µs the request itself costs, so negotiated compression may roughly double CPU per mixed read. This is unmeasured.
- **Near-static large responses:** the country list, language list and UI text catalog account for most response bytes and change only on deploy or i18n refresh. Compressing them once per content version and serving cached bytes, with an `ETag`, would remove most compression work; a lower zstd level for dynamic JSON is a smaller alternative.
- **Benchmark coverage gaps:** the PGO and BOLT benchmark requests identity encoding, uses smaller fixture payloads than production, and reuses 32 connections, so it omits compression, live payload sizes and TLS handshakes (estimated 0.1-0.2 ms of CPU per full handshake on Zen 3). It measures public reads only; photograph uploads and image processing, Minecraft seed sampling, and database writes are unmeasured.
- **Benchmark host differs from deployment:** all optimized-build measurements ran on the workstation, not the miniserver. The About Blog headline of 53,889 requests/s and its results describe four server cores of the 9800X3D, while the same page describes the eight-core miniserver. The wording was left unchanged on November 3 at the owner's request; if revised, lead with the relative PGO and BOLT gains.

## Remaining

Next action, when optimization resumes: measure compression cost by adding a benchmark variant that sends `Accept-Encoding: zstd` and comparing backend CPU per request against identity on the same binaries; then decide on cached precompressed bodies for the near-static responses. Measuring on the miniserver itself needs a client on another machine and a maintenance window, because it is the live host.
