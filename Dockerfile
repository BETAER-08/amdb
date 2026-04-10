FROM rust:1.77-slim-bookworm AS builder
WORKDIR /usr/src/amdb

# rusqlite 등 C 바인딩 라이브러리 컴파일을 위한 필수 도구 설치
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# 런타임에 필요한 인증서 및 라이브러리 추가
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/amdb/target/release/amdb /usr/local/bin/amdb
ENTRYPOINT ["amdb"]