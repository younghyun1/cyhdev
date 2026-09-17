# HTTPS transport latency

The HTTPS listener wraps its Rustls acceptor with axum-server's `NoDelayAcceptor`, which sets `TCP_NODELAY` on each accepted TCP socket before TLS negotiation. Preserve this setting when changing listener construction. HTTP/2 headers and DATA can be emitted separately; leaving Nagle's algorithm enabled can delay response completion while waiting for a TCP acknowledgement. This setting applies to the entire HTTPS listener, including map tiles, APIs, and static assets.

## Evidence

On 2026-09-17, sequential requests over a reused localhost connection to the live server returned HTTP/2 headers in less than 1 ms but completed a 59,571-byte tile response in roughly 43 ms. Reused HTTP/1.1 responses completed in roughly 0.4 ms. Inspection of axum-server 0.8.0 showed that the default Rustls acceptor uses a no-op TCP acceptor and does not enable `TCP_NODELAY`.

An isolated development-profile experiment served an identical 59,571-byte in-memory response with the same Axum/Rustls stack. The only server setting changed between runs was `TCP_NODELAY`. Each measurement used 20 sequential requests over a reused TLS connection; the first request, which establishes the connection, was excluded. Curl negotiated and verified the indicated protocol, and every response returned HTTP 200 and the expected body length.

| TCP_NODELAY | Protocol | Median first byte | Median complete response | p95 complete response |
|---|---|---:|---:|---:|
| Disabled | HTTP/2 | 0.290 ms | 40.981 ms | 41.975 ms |
| Enabled | HTTP/2 | 0.113 ms | 0.152 ms | 0.194 ms |
| Disabled | HTTP/1.1 | 0.048 ms | 0.055 ms | 0.134 ms |
| Enabled | HTTP/1.1 | 0.046 ms | 0.053 ms | 0.156 ms |

This isolates the delay to TCP transport behavior, consistent with Nagle/delayed-ACK interaction, rather than tile cache lookup, disk access, compression, or Minecraft rendering. No packet capture was taken. These are low-load loopback measurements, not internet latency or throughput guarantees. The production listener change still requires the normal website build and deployment; the live process was not restarted during investigation.

## Reproduction

Run `cargo test --locked -p rust-be-template --lib compare_reused_tls_connections -- --ignored --nocapture`. The manual test requires `openssl` and a `curl` build supporting HTTP/2. It creates a temporary self-signed certificate, listens only on loopback ephemeral ports, and shuts down both listeners afterward. Timing results are structured JSON logs, not CI latency thresholds, because operating-system ACK policy and scheduler load vary. Backend library Clippy passes with warnings denied.

The upstream [NoDelayAcceptor documentation](https://docs.rs/axum-server/0.8.0/axum_server/accept/struct.NoDelayAcceptor.html) describes the socket setting. After deployment, repeat sequential HTTP/2 tile requests using one curl process so the TLS connection is actually reused; separate curl processes hide this symptom behind connection setup.
