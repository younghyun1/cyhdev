### Building and running the application

Copy `rust-be-template/.env.example` to `rust-be-template/.env`, replace every placeholder, and keep `CURR_ENV=local`, `HOST_IP=0.0.0.0`, and `PUBLIC_APP_ORIGIN=https://localhost:30737` for the local Compose mapping. The TLS certificate must cover `localhost`. Then run `docker compose --file rust-be-template/compose.yaml up --build` from the repository root. The Dockerfile installs frontend dependencies from `package-lock.json`, builds the frontend in its own stage, and builds the backend from the root `Cargo.lock`. Compose injects the backend `.env` file at runtime; the file is excluded from Git and the Docker build context. The image has no implicit deployment mode, so direct container runs must also set `CURR_ENV` explicitly.

Never pass database, SMTP, object-store, or API credentials as Docker build arguments or bake them into the image. Production deployments must provide the same environment variables through the deployment platform's runtime secret mechanism.

The application will be available at https://localhost:30737. Compose publishes the HTTPS and WebRTC ports on `127.0.0.1` only.

### Runtime image layout

The final `scratch` image runs as UID and GID 65532 with working directory `/bin`. `/bin/server`, the Geo-IP bundles `/bin/new_bundle_ipv4.db` and `/bin/new_bundle_ipv6.db`, and the CA bundle are root-owned and read-only. The only writable paths are `/bin/logs` (daily JSON logs and their zstd archives), `/bin/data/search_index` (the default `SEARCH_INDEX_PATH`), and an empty mode-1777 `/tmp` for upload and photo batch staging. Relative paths in `.env`, such as `./certs/fullchain.pem`, resolve against `/bin`.

Compose runs the container with a read-only root filesystem, all capabilities dropped, and `no-new-privileges`. It mounts `rust-be-template/certs` read-only at `/bin/certs`, a size-bounded tmpfs at `/tmp`, and the named volumes `search-index` and `logs`, which inherit UID 65532 ownership from the image when first created. UID 65532 must be able to traverse that directory and read the certificate chain and private key, for example after `setfacl -R -m u:65532:rX rust-be-template/certs`. A non-default `SEARCH_INDEX_PATH` must point inside a writable mount.

The server binds `HOST_PORT=443` without `CAP_NET_BIND_SERVICE` because Docker sets `net.ipv4.ip_unprivileged_port_start=0` inside each container network namespace. Host networking and runtimes without that default need a `HOST_PORT` above 1023 or the same sysctl.

### Deploying your application to the cloud

Build the optimized deployment image, tagged `cyhdev-backend:dev`, without credentials from the repository root with `cargo xtask image`. The command refuses an EU5 submodule that is uninitialized, moved from its recorded gitlink, or locally modified, uses the digest-pinned builders, and passes `APP_BUILD_EPOCH`, defaulting to the current Git commit timestamp for meaningful, reproducible metadata. An explicitly supplied `SOURCE_DATE_EPOCH` remains the source for that value.

If the deployment uses a different CPU architecture than the development machine, invoke Docker from the repository root with the required `--platform` and `--pull` options.

Then push it to the registry, for example: `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/) documentation for more detail on building and pushing.

### References

* [Docker's Rust guide](https://docs.docker.com/language/rust/)
