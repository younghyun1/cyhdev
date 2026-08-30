# Throughput harness

`cargo xtask throughput` replays the checked-in `public-read-v1` workload against a deterministic in-process fixture, writes `target/throughput/public-read-v1.json`, and fails when the checked-in throughput, error-rate, or p50/p95/p99 latency limits are exceeded. The report records workload, environment, and threshold digests; declared run conditions; observed hardware, kernel, and exact Rust compiler; exact concurrency and buffer bounds; aggregate failures; and a response checksum. Configuration inputs are capped at 1 MiB and saved reports at 16 MiB before JSON deserialization.

The default fixture is a portable regression smoke test, not a backend capacity claim. Establish a hardware-specific backend baseline by copying the environment and threshold files, setting `executor_kind` to `http`, and running:

```bash
cargo xtask throughput run --target http://127.0.0.1:3000 --environment path/to/environment.json --thresholds path/to/http-thresholds.json --output target/throughput/backend.json
```

Only `http://` targets are accepted. Terminate TLS in front of the harness when needed. Targets may include an ASCII base path but not credentials, query strings, fragments, whitespace, or control characters. Each request opens one connection and sends HTTP/1.1 with `Connection: close`; the result therefore includes connection setup and does not model HTTP/2, TLS, connection pooling, WebSockets, request bodies, or authenticated traffic. Workloads accept only bounded GET and HEAD cases, at most 64 workers, one million measured requests, and a 16 MiB response cap.

Recheck a saved report without replaying the workload:

```bash
cargo run --locked --package throughput-harness -- check --report target/throughput/backend.json --thresholds path/to/http-thresholds.json
```

Run comparisons on an idle host under the declared power profile. An HTTP environment file must set `configuration.target` to the exact normalized target reported by the harness. A report contains both the portable declared-environment digest and an observed digest that includes hardware, kernel, compiler, and build profile. Put the baseline report's `environment.observed_digest` in `observed_environment_digest` in the HTTP threshold file; HTTP thresholds reject an omitted or mismatched observed digest. Recalibrate thresholds deliberately when the target, hardware, compiler, build profile, or workload changes; do not compare reports with different workload or environment digests as if they were the same baseline.
