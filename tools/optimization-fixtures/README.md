# Disposable cyhdev optimization fixtures

This workspace Rust binary supplies synthetic PostgreSQL accounts/content/retention data, a bounded S3 object store, TLS/auth SMTP mailbox, signed OIDC provider with PKCE and client authentication, same-user Minecraft terrain sockets, squaremap files, and authenticated management acknowledgements. It does not contact a Minecraft server. Every recipient uses `example.test`; generated credentials and seed material are synthetic.

For an automatically managed campaign, run `./build_pgo_and_bolt.sh` from a clean committed workspace. It prepares a fresh private runtime, isolated PostgreSQL 18 container, inventory, and unique config, supervises the providers, runs the optimization campaign, and removes its owned processes/container afterward. See the [optimization runbook](../optimization/README.md) for prerequisites, resource limits, and retained failure diagnostics. The following instructions apply to manually managed fixtures.

Use the dedicated PostgreSQL listener at `127.0.0.1:35432` with user `optimization_fixture`, a password beginning with `optimization-fixture-`, and a database named `cyhdev_optimization_*`. No socket/query overrides or other database listener is admitted. The database role needs create/drop database privileges in this disposable instance. PostgreSQL client tools, OpenSSL, the checked-in public Geo-IP bundles, locked frontend dependencies, and Chromium must be available.

Run from the workspace root:

```bash
cargo build --locked --release --package optimization-fixtures
mkdir -m 700 /tmp/cyh-opt-example
export CYHDEV_OPT_DISPOSABLE=1
export DB_URL=postgres://optimization_fixture:optimization-fixture-example@127.0.0.1:35432/cyhdev_optimization_example
target/release/optimization-fixtures prepare /tmp/cyh-opt-example
target/release/optimization-fixtures campaign /tmp/cyh-opt-example
source /tmp/cyh-opt-example/environment.sh
target/release/optimization-fixtures serve /tmp/cyh-opt-example
```

Keep `serve` running in its own terminal while executing the [optimization campaign](../optimization/README.md). Preparation requires a fresh runtime receipt and fresh snapshot database; it never overwrites an existing snapshot. The generated environment contains synthetic credentials and private TLS/OIDC keys. Certificates expire after two days, so prepare new inputs for a later campaign. The generator writes ignored `target/optimization-inputs/campaign.json` and `config.json`; generate the source inventory first and choose a fresh campaign name before each optimized run.

`campaign RUNTIME OUTPUT_DIR NAME CPU` writes the config/workload beside an inventory in a supplied absolute directory under `target/optimization-inputs`; the managed wrapper uses this form for unique campaigns. The two-argument form retains the manual default directory and name.

The reset hook hashes the golden PostgreSQL dump, verifies asset/key/Geo-IP identities, recreates only the admitted runtime database from the snapshot, checks clone equality, and resets search/object/mail/provider state. Database seeding is transactional. The dump is capped at 128 MiB; files are capped at 512 MiB. Object storage admits at most 256 objects and 128 MiB; mail admits at most 64 bounded messages. Terrain and management connections admit four concurrent clients with bounded messages and deadlines. Missing or invalid visibility state fails closed. SIGINT/SIGTERM stops the fixture listeners and removes their own Unix socket paths.

The generated workload exercises all declared HTTP operations, every route at desktop/mobile sizes with both themes and English/Korean, and the functional scenario policy. It performs real password sessions and signed OIDC exchanges, consumes captured mail, waits for background processing, sends actual fake-device audio/video through the RTC backend, and checks negative authorization/capability/revision paths. Fresh login requests are paced to respect the application's IP rate limit. An additional 1,000 successful public reads weight normal traffic during training. The native `oha` benchmark uses nine samples of 100,000 public reads, 32 pooled HTTPS connections, four worker threads and 10,000 warmup requests, with fresh snapshot clones for each variant. Set `CYHDEV_OHA` to the installed executable's absolute path; see the optimization runbook for affinity and client-headroom checks.

Development diagnostics accept `CYHDEV_OPT_DIAGNOSTIC=pages|workflows` and optional workflow group `content|media|protocols|minecraft|browser|system`. They require `CYHDEV_OPT_STAGE=development` and deliberately record failed coverage, so partial checks cannot establish optimized acceptance. Use the full workload without either diagnostic variable for training and final verification. Fixture dimensions, tiny synthetic media, and localhost latency are functional inputs; they do not represent production datasets or capacity.

The campaign driver selects the fixed [optimization request policy](../optimization/request-limits.json) only after validating disposable fixtures. Backend startup independently requires local deployment, numeric loopback listeners/origin/database, the disposable marker, the fixture database namespace, and no trusted proxies. The finite 16,384-request burst and microsecond refill accommodate cold browser assets and native traffic while retaining the client-table cap; public defaults and authentication limits remain enforced. The policy contributes to benchmark identity. The native mix uses the raw server-health response; fresh database latency remains covered by the functional host scenario and retains its four-probe concurrency cap. Generate fresh campaign inputs after policy or workload changes.
