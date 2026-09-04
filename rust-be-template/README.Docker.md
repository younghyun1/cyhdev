### Building and running the application

Copy `rust-be-template/.env.example` to `rust-be-template/.env`, replace every placeholder, and keep `CURR_ENV=local`, `HOST_IP=0.0.0.0`, and `PUBLIC_APP_ORIGIN=https://localhost:30737` for the local Compose mapping. The TLS certificate must cover `localhost`. Then run `docker compose --file rust-be-template/compose.yaml up --build` from the repository root. The Dockerfile installs frontend dependencies from `package-lock.json`, builds the frontend in its own stage, and builds the backend from the root `Cargo.lock`. Compose injects the backend `.env` file at runtime; the file is excluded from Git and the Docker build context. The image has no implicit deployment mode, so direct container runs must also set `CURR_ENV` explicitly.

Never pass database, SMTP, object-store, or API credentials as Docker build arguments or bake them into the image. Production deployments must provide the same environment variables through the deployment platform's runtime secret mechanism.

The application will be available at https://localhost:30737.

### Deploying your application to the cloud

Build the development image without credentials from the repository root with `cargo xtask image`. The command verifies the digest-pinned nightly builder and passes `APP_BUILD_EPOCH`, defaulting to the current Git commit timestamp for meaningful, reproducible metadata. An explicitly supplied `SOURCE_DATE_EPOCH` remains the source for that value.

If the deployment uses a different CPU architecture than the development machine, invoke Docker from the repository root with the required `--platform` and `--pull` options.

Then push it to the registry, for example: `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/) documentation for more detail on building and pushing.

### References

* [Docker's Rust guide](https://docs.docker.com/language/rust/)
