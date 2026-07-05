# Generic builder for any Rust app in the workspace. Pass the binary via BIN.
#   docker build -f docker/rust.Dockerfile --build-arg BIN=omm-gateway -t omm-gateway .
# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
ARG BIN
WORKDIR /build
COPY . .
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release -p "${BIN}" && \
    cp "target/release/${BIN}" /usr/local/bin/app

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Run unprivileged.
RUN useradd --system --uid 10001 omm
COPY --from=builder /usr/local/bin/app /usr/local/bin/app
USER omm
ENTRYPOINT ["/usr/local/bin/app"]
