FROM rust:1.77-slim-bookworm AS builder
WORKDIR /usr/src/amdb
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/amdb/target/release/amdb /usr/local/bin/amdb
ENTRYPOINT ["amdb"]