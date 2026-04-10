FROM rust:1.77-slim-bookworm AS builder
WORKDIR /usr/src/amdb

# fastembed(ort) 및 C 바인딩 컴파일을 위한 필수 도구 (cmake, clang 추가)
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential cmake clang && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# 런타임 인증서 추가
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/amdb/target/release/amdb /usr/local/bin/amdb
ENTRYPOINT ["amdb"]