# syntax=docker/dockerfile:1

ARG APP_NAME=rust-be-template

# Build the frontend from the committed npm lockfile inside the image build.
# This prevents an ambient local dist directory from changing the backend image.
FROM node:24-alpine AS frontend
WORKDIR /workspace/solid-csr-spa-template
COPY solid-csr-spa-template/package.json solid-csr-spa-template/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY solid-csr-spa-template/ ./
RUN npm run build

# The rolling tag intentionally tracks the latest nightly toolchain. Root image
# commands use --pull so a cached base cannot pin an older nightly.
FROM rustlang/rust:nightly-alpine AS rust-base
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-musl
WORKDIR /workspace

# Install native build dependencies required by the bundled C libraries.
RUN apk add --no-cache clang lld musl-dev git ca-certificates postgresql-dev upx zstd-static pkgconf make perl \
    && rustup component add rust-src

# Validate the current build context, lockfile, rolling nightly, and rust-src
# installation without compiling the release-only runtime image.
FROM rust-base AS smoke
COPY . ./
RUN cargo metadata --locked --no-deps --format-version 1 >/dev/null

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
        -Z build-std-features=backtrace && \
    upx --lzma --best "./target/$RUST_TARGET/release/$APP_NAME" && \
    cp "./target/$RUST_TARGET/release/$APP_NAME" /bin/server

# --- Final Stage ---
FROM scratch AS final

# Copy the server executable from the build stage
COPY --from=build /bin/server /bin/
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy database bundle files
COPY rust-be-template/new_bundle_ipv4.db /bin/
COPY rust-be-template/new_bundle_ipv6.db /bin/

# Set non-secret defaults. Inject database and service credentials at runtime.
ENV CURR_ENV="dev"
ENV HOST_IP="0.0.0.0"
ENV HOST_PORT="443"
ENV SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"

# Expose the application port
EXPOSE 443

# Expose the WebRTC SFU UDP range (set RTC_UDP_PORT_START to match when RTC is enabled)
EXPOSE 3478-3541/udp

# Set the command to run the application
CMD ["/bin/server"]
