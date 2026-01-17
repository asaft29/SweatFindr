FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/backend/ .

RUN mkdir -p notification-service/src && \
    echo "fn main() {}" > notification-service/src/main.rs && \
    mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p email-service/src && \
    echo "fn main() {}" > email-service/src/main.rs

RUN cargo build --release --manifest-path notification-service/Cargo.toml

RUN rm -rf notification-service/src

COPY services/backend/notification-service/src ./notification-service/src
COPY services/backend/notification-service/build.rs ./notification-service/build.rs
COPY services/backend/notification-service/proto ./notification-service/proto

RUN touch notification-service/src/main.rs && \
    cargo build --release --manifest-path notification-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/notification-service /usr/local/bin/notification-service

EXPOSE 8080

CMD ["notification-service"]
