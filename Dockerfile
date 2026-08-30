ARG APP_NAME=rust-be-template

# --- Build Stage ---
FROM rustlang/rust:nightly-alpine AS build
ARG APP_NAME
ARG RUST_TARGET=x86_64-unknown-linux-musl
WORKDIR /app

# Install build dependencies, including tools for vendored OpenSSL
RUN apk add --no-cache clang lld musl-dev git ca-certificates postgresql-dev upx zstd-static pkgconf make perl \
    && rustup component add rust-src

# Rebuild the standard library with the release profile from Cargo.toml so the
# application and sysroot share the same optimization, LTO, and panic settings.
RUN --mount=type=bind,source=.cargo,target=.cargo \
    --mount=type=bind,source=rust-toolchain.toml,target=rust-toolchain.toml \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=bind,source=src,target=src,rw \
    --mount=type=bind,source=fe,target=fe \
    --mount=type=bind,source=i18n,target=i18n \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --target "$RUST_TARGET" \
        -Z build-std=core,alloc,std,panic_unwind \
        -Z build-std-features=backtrace && \
    upx --lzma --best "./target/$RUST_TARGET/release/$APP_NAME" && \
    cp "./target/$RUST_TARGET/release/$APP_NAME" /bin/server

# --- Final Stage ---
FROM scratch AS final

# Copy the server executable from the build stage
COPY --from=build /bin/server /bin/

# Copy database bundle files
COPY new_bundle_ipv4.db /bin/
COPY new_bundle_ipv6.db /bin/

# Set non-secret defaults. Inject database and service credentials at runtime.
ENV CURR_ENV="dev"
ENV HOST_IP="127.0.0.1"
ENV HOST_PORT="443"

# Expose the application port
EXPOSE 443

# Expose the WebRTC SFU UDP range (set RTC_UDP_PORT_START to match when RTC is enabled)
EXPOSE 3478-3541/udp

# Set the command to run the application
CMD ["/bin/server"]
