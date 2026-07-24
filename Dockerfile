FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/amdb

RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential cmake clang && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release -j 2

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY --from=builder /usr/src/amdb/target/release/amdb /usr/local/bin/amdb
ENTRYPOINT ["amdb", "serve"]