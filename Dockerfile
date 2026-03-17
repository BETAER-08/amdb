# Base stage: install cargo-chef
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Planner stage: only needs manifests to generate recipe.json
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage: compile dependencies then source
FROM chef AS builder
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    mkdir /out && \
    cp /app/target/release/amdb /out/amdb

FROM gcr.io/distroless/cc-debian12 AS runtime

COPY --chown=nonroot:nonroot --from=builder /out/amdb /usr/local/bin/amdb

USER nonroot
WORKDIR /data

LABEL org.opencontainers.image.source="https://github.com/BETAER-08/amdb"
LABEL org.opencontainers.image.license="MIT"

ENTRYPOINT ["amdb"]
