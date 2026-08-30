# syntax=docker/dockerfile:1

ARG APP_NAME=rust-be-template
ARG RUST_DOCKER_TAG=nightly

# Build the frontend from the committed npm lockfile inside the image build.
# This prevents an ambient local dist directory from changing the backend image.
FROM node:24-alpine AS frontend
WORKDIR /workspace/solid-csr-spa-template
RUN apk add --no-cache gzip zstd
COPY solid-csr-spa-template/package.json solid-csr-spa-template/package-lock.json solid-csr-spa-template/.npmrc ./
RUN npm ci --legacy-peer-deps --no-audit --no-fund
COPY solid-csr-spa-template/ ./
RUN npm run build \
    && find ./dist -type f ! -iname '*.png' ! -iname '*.jpg' ! -iname '*.gz' ! -iname '*.zst' \
        -exec sh -c 'gzip -9 -c "$1" > "$1.gz"; zstd --ultra -22 -q -c "$1" > "$1.zst"' _ {} \;

# The compatibility artifact preserves the original build.sh ABI and output:
# glibc, Zen 3 tuning, no UPX, and a host-visible Cargo release path.
FROM rustlang/rust:${RUST_DOCKER_TAG} AS host-release
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-gnu
ARG RUST_TARGET_CPU=znver3
ARG SOURCE_DATE_EPOCH=0
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
RUN --mount=type=cache,target=/workspace/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    RUSTFLAGS="-C target-cpu=${RUST_TARGET_CPU}" \
    CFLAGS="-march=${RUST_TARGET_CPU} -O3" \
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

# The rolling tag intentionally tracks the latest nightly toolchain. Root image
# commands use --pull so a cached base cannot pin an older nightly.
FROM rustlang/rust:nightly-alpine AS rust-base
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
RUN --mount=type=cache,target=/workspace/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --package "$APP_NAME" && \
    cp "./target/debug/$APP_NAME" /bin/server

# A runnable development-profile layout used by the Wave 8 image smoke gate.
# Runtime secrets remain injected by the caller; the build only verifies that
# the executable and immutable Geo-IP assets land at their runtime paths.
FROM alpine:3.23 AS smoke
RUN apk add --no-cache ca-certificates libgcc libpq
WORKDIR /app
COPY --from=smoke-build /bin/server /app/server
COPY rust-be-template/new_bundle_ipv4.db rust-be-template/new_bundle_ipv6.db /app/
RUN test -x /app/server \
    && test -s /app/new_bundle_ipv4.db \
    && test -s /app/new_bundle_ipv6.db \
    && (ldd /app/server >/tmp/server-ldd.txt 2>&1 || true) \
    && ! grep -q "not found" /tmp/server-ldd.txt
ENV HOST_IP="0.0.0.0"
ENV HOST_PORT="443"
ENV SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"
CMD ["/app/server"]

FROM rust-base AS build
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-musl
ARG SOURCE_DATE_EPOCH=0
COPY . ./
COPY --from=frontend /workspace/solid-csr-spa-template/dist/ solid-csr-spa-template/dist/

# Rebuild the standard library with the release profile from Cargo.toml so the
# application and sysroot share the same optimization, LTO, and panic settings.
RUN --mount=type=cache,target=/workspace/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --package "$APP_NAME" --target "$RUST_TARGET" \
        -Z build-std=core,alloc,std,panic_unwind \
        -Z build-std-features=backtrace,panic-unwind && \
    upx --lzma --best "./target/$RUST_TARGET/release/$APP_NAME" && \
    cp "./target/$RUST_TARGET/release/$APP_NAME" /bin/server

# --- Final Stage ---
FROM scratch AS final

WORKDIR /bin

# Copy the server executable from the build stage
COPY --from=build /bin/server /bin/
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy database bundle files
COPY rust-be-template/new_bundle_ipv4.db /bin/
COPY rust-be-template/new_bundle_ipv6.db /bin/

# Set non-secret defaults. Inject database and service credentials at runtime.
ENV HOST_IP="0.0.0.0"
ENV HOST_PORT="443"
ENV SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"

# Expose the application port
EXPOSE 443

# Expose the WebRTC SFU UDP range (set RTC_UDP_PORT_START to match when RTC is enabled)
EXPOSE 3478-3541/udp

# Set the command to run the application
CMD ["/bin/server"]
