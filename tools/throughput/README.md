# Throughput harness

`cargo xtask throughput` compiles and executes `throughput-harness` with the workspace release profile. It replays the checked-in `public-read-v1` workload against a deterministic in-process fixture, writes `target/throughput/public-read-v1.json`, and fails when the checked-in throughput, error-rate, or p50/p95/p99 latency limits are exceeded. The report records workload, environment, and threshold digests; declared run conditions; observed hardware, kernel, and exact Rust compiler; exact concurrency and buffer bounds; aggregate failures; and a response checksum. Configuration inputs are capped at 1 MiB and saved reports at 16 MiB before JSON deserialization.

The default fixture is a portable regression smoke test, not a backend capacity claim. A real baseline has two phases. `record` measures the backend without thresholds and deliberately emits no verdict; a normal run then enforces reviewed thresholds pinned to that machine and configuration.

## Local baseline preparation

Use a disposable PostgreSQL 18 database and non-production credentials. Never put a database URL, password, token, or other secret in the declared environment JSON because the complete declaration is copied into every report. The backend task runs with `rust-be-template` as its working directory so its ignored `.env`, Geo-IP bundles, certificates, and search index resolve consistently. From the repository root, copy [the environment template](config/http-local.example.json) into the ignored `target/throughput` directory. In the same shell that will start the backend, read and export the disposable database URL without placing it in shell history, then compute the exact source, database, Geo, and bridge evidence:

```bash
set -euo pipefail
mkdir -p target/throughput
cp tools/throughput/config/http-local.example.json target/throughput/backend-environment.json
read -rsp "Disposable DB_URL: " DB_URL && printf '\n'
export DB_URL
working_tree_digest=$(
  { git rev-parse HEAD; git diff --binary HEAD --; git ls-files --others --exclude-standard -z | sort -z | xargs -0 -r sha256sum; } \
    | sha256sum | awk '{print "sha256:" $1}'
)
database_server_version=$(PGDATABASE="$DB_URL" psql -Atqc "select current_setting('server_version')")
database_schema_digest=$(PGDATABASE="$DB_URL" pg_dump --schema-only --no-owner --no-privileges --restrict-key=0123456789abcdef0123456789abcdef \
  | sha256sum | awk '{print "sha256:" $1}')
database_dataset_digest=$(PGDATABASE="$DB_URL" pg_dump --data-only --no-owner --no-privileges --restrict-key=0123456789abcdef0123456789abcdef \
  | sha256sum | awk '{print "sha256:" $1}')
geo_ipv4_digest=$(sha256sum rust-be-template/new_bundle_ipv4.db | awk '{print "sha256:" $1}')
geo_ipv6_digest=$(sha256sum rust-be-template/new_bundle_ipv6.db | awk '{print "sha256:" $1}')
printf '%s\n' "$working_tree_digest" "$database_server_version" "$database_schema_digest" \
  "$database_dataset_digest" "$geo_ipv4_digest" "$geo_ipv6_digest"
openssl version
socat -V 2>&1 | head -n 1
```

Copy those outputs into the matching fields without including `DB_URL`. The `PGDATABASE` commands use the exact URL exported to the backend, rather than whichever local cluster `psql` would otherwise choose. Repeat every digest immediately before regression runs; any mismatch requires restoring the recorded inputs or deliberately recalibrating.

Build both measured executables before calibration. `./build.sh` produces the optimized `znver3` GNU backend artifact with the workspace release profile; the separate Cargo command ensures the harness is already compiled with the same workspace release profile before any measurement command starts:

```bash
./build.sh
cargo build --locked --release --package throughput-harness
```

Start the optimized backend artifact in one terminal. Existing environment values provide the disposable database and runtime dependencies; running from `rust-be-template` preserves its backend-relative `.env`, Geo-IP bundle, certificate, and search-index paths. The explicit listener values keep the measurement on loopback and outside privileged ports:

```bash
cd rust-be-template
HOST_IP=127.0.0.1 HOST_PORT=8443 ../target/x86_64-unknown-linux-gnu/release/rust-be-template
```

After the HTTPS health endpoint succeeds, bridge a plain loopback HTTP listener to the backend in a second terminal. Keep this exact bridge in place for calibration and regression runs because its TLS and process-per-connection overhead is part of the recorded topology:

```bash
curl --fail --silent --show-error --insecure https://127.0.0.1:8443/api/healthcheck/server
socat TCP-LISTEN:3000,bind=127.0.0.1,reuseaddr,fork OPENSSL:127.0.0.1:8443,verify=0,snihost=localhost
```

Run at least five threshold-free calibrations on an idle host. Use distinct output paths and keep all declared and observed environment digests identical:

```bash
cargo xtask throughput record --target http://127.0.0.1:3000 --environment target/throughput/backend-environment.json --output target/throughput/backend-calibration-1.json
```

Repeat with output suffixes `2` through `5`. Review every report. Each must have zero failures and must record the same workload digest, declared environment digest, observed environment digest, executor target, resolved address, compiled profile, and hardware fields. Set the initial minimum throughput to 90 percent of the lowest observed requests per second. Set each latency maximum to 110 percent of the highest observed p50, p95, or p99, rounded up. Keep maximum error rate at zero. These are starting regression budgets, not capacity objectives.

Create `target/throughput/backend-thresholds.json` with schema version 2, using `config/fixture-thresholds.json` as the shape. Copy the workload and environment names/digests, `configuration.compiled_profile`, `configuration.implementation_digest`, and `executor.resolved_address` from the calibration reports; set `executor_kind` to `http`. Then prove the threshold file with a fresh run:

```bash
cargo xtask throughput run --target http://127.0.0.1:3000 --environment target/throughput/backend-environment.json --thresholds target/throughput/backend-thresholds.json --output target/throughput/backend-http.json
```

Only `http://` targets are accepted. Terminate TLS in front of the harness when needed. Targets may include an ASCII base path but not credentials, query strings, fragments, whitespace, or control characters. Each request opens one connection and sends HTTP/1.1 with `Connection: close`; the result therefore includes connection setup and does not model HTTP/2, TLS, connection pooling, WebSockets, request bodies, or authenticated traffic. `timeout_ms` is one absolute connect/write/read deadline. Workloads accept only bounded GET and HEAD cases, at most 64 workers, one million measured requests, a 16 MiB per-response cap, and a 64 MiB aggregate concurrent response-buffer cap.

Recheck a saved calibration or thresholded report without replaying the workload:

```bash
cargo run --locked --release --package throughput-harness -- check --report target/throughput/backend-http.json --thresholds target/throughput/backend-thresholds.json
```

Run comparisons on an idle host under the declared power profile. An HTTP environment file must set `configuration.target` to the exact normalized target reported by the harness. A report contains both the canonical declared-environment digest and an observed digest that includes hardware, kernel, compiler, build profile, resolved socket address, normalized target, and a build-time SHA-256 digest of the harness sources, manifests, lockfile, and toolchain declaration. HTTP thresholds require and enforce the observed, implementation, and resolved-address evidence. Recalibrate thresholds deliberately when any recorded input changes; do not compare reports with different evidence as if they were the same baseline.

## Wave 8 gate

Leave the backend and bridge running, then export the pinned inputs before the repository review:

```bash
THROUGHPUT_HTTP_TARGET=http://127.0.0.1:3000 \
THROUGHPUT_HTTP_ENVIRONMENT=target/throughput/backend-environment.json \
THROUGHPUT_HTTP_THRESHOLDS=target/throughput/backend-thresholds.json \
THROUGHPUT_HTTP_OUTPUT=target/throughput/backend-http.json \
cargo xtask final-review
```
