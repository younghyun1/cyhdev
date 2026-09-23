# syntax=docker/dockerfile:1

ARG APP_NAME=rust-be-template
# Keep local cache keys stable until these reviewed builder revisions are
# intentionally advanced. The tags retain their human-readable provenance.
ARG RUST_DOCKER_TAG=nightly@sha256:dfd905886275b22be070dad4e8f2c2264bae8341ba7a4b2ceea370249190b044
ARG RUST_ALPINE_DOCKER_TAG=nightly-alpine@sha256:ad0a86a1629fcf9e77b8b6d92a7803e16ce011a1c80a9602786abd5cb225d615
ARG NODE_DOCKER_TAG=24-alpine@sha256:e67514e5d0f6c46656005e1b693b2ec9d52e80b641307de684d4a015ba7a4eaf
ARG ALPINE_DOCKER_TAG=3.23@sha256:85fe1e81d6758c208f3e1eed4338a1997e19d4be002d4dd32d3100c9a8c010a0

# Build the pinned EU5 source revision as a browser-native ES module package.
# wasm-pack would otherwise download wasm-bindgen and binaryen release archives
# with no integrity check. wasm-bindgen is built from crates.io, whose index
# checksums cover every locked crate, and wasm-opt comes from Alpine's signed
# binaryen package. `--mode no-install` makes wasm-pack use those PATH binaries:
# it fails when the locked wasm-bindgen version is absent, and it would skip
# wasm-opt silently, so the build step checks both first.
FROM rustlang/rust:${RUST_ALPINE_DOCKER_TAG} AS eu5-wasm
ARG WASM_PACK_VERSION=0.15.0
ENV RUSTUP_TOOLCHAIN="${RUST_VERSION}"
RUN apk add --no-cache binaryen clang lld musl-dev git ca-certificates make perl \
    && rustup target add wasm32-unknown-unknown --toolchain "$RUSTUP_TOOLCHAIN" \
    && cargo install wasm-pack --version "$WASM_PACK_VERSION" --locked
# Must equal the wasm-bindgen version in vendor/eu5-location-filter/Cargo.lock;
# the build step rejects a mismatch after the submodule moves.
ARG WASM_BINDGEN_VERSION=0.2.127
RUN cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --locked
WORKDIR /workspace/vendor/eu5-location-filter
COPY vendor/eu5-location-filter/ ./
RUN --mount=type=cache,id=cyhdev-eu5-target,target=/workspace/vendor/eu5-location-filter/target,sharing=locked \
    --mount=type=cache,id=cyhdev-eu5-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cyhdev-eu5-cargo-git,target=/usr/local/cargo/git/db,sharing=locked \
    locked_bindgen="$(sed -n '/^name = "wasm-bindgen"$/{n;s/^version = "\(.*\)"$/\1/p;}' Cargo.lock)" \
    && if [ "$locked_bindgen" != "$WASM_BINDGEN_VERSION" ]; then \
        echo "EU5 Cargo.lock requires wasm-bindgen ${locked_bindgen:-<missing>}; set WASM_BINDGEN_VERSION to match" >&2; \
        exit 1; \
    fi \
    && wasm-bindgen --version \
    && wasm-opt --version \
    && wasm-pack build --release --mode no-install --target web --no-pack --no-typescript -- \
        --locked --no-default-features --features web \
    && test -s pkg/eu5_location_filter.js \
    && test "$(od -An -tx1 -N4 pkg/eu5_location_filter_bg.wasm | tr -d ' \n')" = "0061736d"

# Build the frontend from the committed npm lockfile inside the image build.
# This prevents an ambient local dist directory from changing the backend image.
FROM node:${NODE_DOCKER_TAG} AS frontend-source
WORKDIR /workspace/solid-csr-spa-template
RUN apk add --no-cache gzip zstd
COPY solid-csr-spa-template/package.json solid-csr-spa-template/package-lock.json solid-csr-spa-template/.npmrc ./
RUN --mount=type=cache,id=cyhdev-npm-cache,target=/root/.npm,sharing=locked \
    npm ci --legacy-peer-deps --no-audit --no-fund
COPY solid-csr-spa-template/ ./
# Browser locale chunks import these public catalogs from the backend-owned source.
COPY rust-be-template/i18n/ui/ /workspace/rust-be-template/i18n/ui/

# Keep source-only frontend checks independent of the optimized EU5 build.
FROM frontend-source AS frontend
COPY --from=eu5-wasm /workspace/vendor/eu5-location-filter/web/index.html ./public/eu5-locations-db/app/index.html
COPY --from=eu5-wasm /workspace/vendor/eu5-location-filter/pkg/eu5_location_filter.js ./public/eu5-locations-db/app/pkg/eu5_location_filter.js
COPY --from=eu5-wasm /workspace/vendor/eu5-location-filter/pkg/eu5_location_filter_bg.wasm ./public/eu5-locations-db/app/pkg/eu5_location_filter_bg.wasm
ARG APP_BUILD_EPOCH
ARG SOURCE_DATE_EPOCH
RUN npm run build \
    && find ./dist -type f ! -iname '*.png' ! -iname '*.jpg' ! -iname '*.gz' ! -iname '*.zst' \
        -exec sh -c 'gzip -9 -c "$1" > "$1.gz"; zstd --ultra -22 -q -c "$1" > "$1.zst"' _ {} \; \
    && find ./dist/eu5-locations-db/app -type f ! -name '*.gz' -delete \
    && test -z "$(find ./dist/eu5-locations-db/app -type f ! -name '*.gz' -print -quit)" \
    && find ./dist/eu5-locations-db/app -type f -name '*.gz' -exec gzip -t {} +

# The compatibility artifact preserves the original build.sh ABI and output:
# glibc, Zen 3 tuning, no UPX, and a host-visible Cargo release path.
FROM rustlang/rust:${RUST_DOCKER_TAG} AS host-release
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-gnu
ARG RUST_TARGET_CPU=znver3
# Bind Cargo and rustup to the dated nightly contained by the freshly pulled
# image. Otherwise rust-toolchain.toml selects a second, mutable nightly alias.
ENV RUSTUP_TOOLCHAIN="${RUST_VERSION}"
WORKDIR /workspace
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates clang git gzip libpq-dev libzstd-dev lld make perl pkg-config zstd \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add rust-src --toolchain "$RUSTUP_TOOLCHAIN"
COPY . ./
COPY --from=frontend /workspace/solid-csr-spa-template/dist/ solid-csr-spa-template/dist/
ARG APP_BUILD_EPOCH
ARG SOURCE_DATE_EPOCH
RUN --mount=type=cache,target=/workspace/target/,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
    build_epoch="${APP_BUILD_EPOCH:-${SOURCE_DATE_EPOCH:-$(date +%s)}}" \
    && RUSTFLAGS="-C target-cpu=${RUST_TARGET_CPU}" \
    CFLAGS="-march=${RUST_TARGET_CPU} -O3" \
    APP_BUILD_EPOCH="$build_epoch" \
    cargo build --locked --release --package "$APP_NAME" --target "$RUST_TARGET" \
        -Z build-std=core,alloc,std,panic_unwind \
        -Z build-std-features=backtrace,panic-unwind \
    && ldd_output="$(ldd "./target/$RUST_TARGET/release/$APP_NAME")" \
    && printf '%s\n' "$ldd_output" \
    && ! printf '%s\n' "$ldd_output" | grep -q 'not found' \
    && mkdir -p /out \
    && cp "./target/$RUST_TARGET/release/$APP_NAME" "/out/$APP_NAME"

FROM scratch AS artifact
COPY --from=host-release /out/ /

# The reviewed digest above keeps compiler fingerprints stable across local
# builds. Updating the tag or digest is an explicit toolchain upgrade.
FROM rustlang/rust:${RUST_ALPINE_DOCKER_TAG} AS rust-base
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-musl
ENV RUSTUP_TOOLCHAIN="${RUST_VERSION}"
WORKDIR /workspace

# Install native build dependencies required by the bundled C libraries.
RUN apk add --no-cache clang lld musl-dev git ca-certificates postgresql-dev upx zstd-static pkgconf make perl \
    && rustup component add rust-src --toolchain "$RUSTUP_TOOLCHAIN"

# Compile the current backend in the development profile. This stage also
# consumes the frontend output because the backend embeds that directory.
FROM rust-base AS smoke-build
ARG APP_NAME
COPY . ./
# Frontend correctness has its own Wave 8 gate. A deterministic placeholder is
# sufficient to compile and lay out the backend development image without
# coupling this smoke target to production frontend bundling.
RUN mkdir -p solid-csr-spa-template/dist \
    && printf '%s\n' '<!doctype html><title>backend smoke</title>' > solid-csr-spa-template/dist/index.html
RUN --mount=type=cache,target=/workspace/target/,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
    cargo build --locked --package "$APP_NAME" && \
    cp "./target/debug/$APP_NAME" /bin/server

# A runnable development-profile layout used by the Wave 8 image smoke gate.
# Runtime secrets remain injected by the caller; the build only verifies that
# the executable and immutable Geo-IP assets land at their runtime paths.
FROM alpine:${ALPINE_DOCKER_TAG} AS smoke
RUN apk add --no-cache ca-certificates libgcc libpq
WORKDIR /app
COPY --from=smoke-build /bin/server /app/server
COPY --chmod=0444 rust-be-template/new_bundle_ipv4.db rust-be-template/new_bundle_ipv6.db /app/
RUN test -x /app/server \
    && test -s /app/new_bundle_ipv4.db \
    && test -s /app/new_bundle_ipv6.db \
    && (ldd /app/server >/tmp/server-ldd.txt 2>&1 || true) \
    && ! grep -q "not found" /tmp/server-ldd.txt \
    && rm /tmp/server-ldd.txt \
    && mkdir -p /app/logs /app/data/search_index \
    && chown 65532:65532 /app/logs /app/data/search_index \
    && chmod 0700 /app/logs /app/data/search_index
ENV HOST_IP="0.0.0.0"
ENV HOST_PORT="443"
ENV SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"
# Match the final image's unprivileged user and write paths so this layout
# exercises the same permissions.
USER 65532:65532
CMD ["/app/server"]

FROM rust-base AS build
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-musl
COPY . ./
COPY --from=frontend /workspace/solid-csr-spa-template/dist/ solid-csr-spa-template/dist/
ARG APP_BUILD_EPOCH
ARG SOURCE_DATE_EPOCH

# Rebuild the standard library with the release profile from Cargo.toml so the
# application and sysroot share the same optimization, LTO, and panic settings.
RUN --mount=type=cache,target=/workspace/target/,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
    build_epoch="${APP_BUILD_EPOCH:-${SOURCE_DATE_EPOCH:-$(date +%s)}}" \
    && APP_BUILD_EPOCH="$build_epoch" \
    cargo build --locked --release --package "$APP_NAME" --target "$RUST_TARGET" \
        -Z build-std=core,alloc,std,panic_unwind \
        -Z build-std-features=backtrace,panic-unwind && \
    upx --lzma --best "./target/$RUST_TARGET/release/$APP_NAME" && \
    cp "./target/$RUST_TARGET/release/$APP_NAME" /bin/server

# Empty runtime directories for the scratch image, which has no shell to create
# them. COPY --from preserves these owners and modes, including the sticky bit.
# The backend resolves ./logs (tracing appender and zstd log compression) and
# ./data/search_index (default SEARCH_INDEX_PATH) against WORKDIR /bin, and
# tempfile uploads plus photo batch staging use /tmp.
FROM alpine:${ALPINE_DOCKER_TAG} AS runtime-layout
RUN mkdir -p /runtime/tmp /runtime/bin/logs /runtime/bin/data/search_index \
    && chmod 1777 /runtime/tmp \
    && chown 65532:65532 /runtime/bin/logs /runtime/bin/data/search_index \
    && chmod 0700 /runtime/bin/logs /runtime/bin/data/search_index

# --- Final Stage ---
FROM scratch AS final

WORKDIR /bin

COPY --from=runtime-layout /runtime/ /

# Root-owned and read-only for the runtime user: the executable, the CA bundle,
# and the Geo-IP bundles, whose modes are set explicitly so a restrictive
# checkout umask cannot make them unreadable.
COPY --from=build --chmod=0555 /bin/server /bin/server
COPY --from=build --chmod=0444 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --chmod=0444 rust-be-template/new_bundle_ipv4.db rust-be-template/new_bundle_ipv6.db /bin/

# Set non-secret defaults. Inject database and service credentials at runtime.
ENV HOST_IP="0.0.0.0"
ENV HOST_PORT="443"
ENV SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"

# Run as a numeric unprivileged user; scratch has no /etc/passwd to resolve a
# name. Binding 443 without CAP_NET_BIND_SERVICE relies on Docker setting
# net.ipv4.ip_unprivileged_port_start=0 in each container network namespace,
# which it does for every network mode except host. Under host networking or a
# runtime without that default, set HOST_PORT above 1023 or set the sysctl.
USER 65532:65532

# Expose the application port
EXPOSE 443

# Expose the WebRTC SFU UDP range (set RTC_UDP_PORT_START to match when RTC is enabled)
EXPOSE 3478-3541/udp

# Set the command to run the application
CMD ["/bin/server"]
