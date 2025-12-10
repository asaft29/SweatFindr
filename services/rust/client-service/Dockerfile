FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/rust/Cargo.toml ./Cargo.toml
COPY services/rust/Cargo.lock ./Cargo.lock

COPY services/rust/client-service/Cargo.toml ./client-service/Cargo.toml
COPY services/rust/event-service/Cargo.toml ./event-service/Cargo.toml
COPY services/rust/auth-service/Cargo.toml ./auth-service/Cargo.toml
COPY services/rust/common/Cargo.toml ./common/Cargo.toml

COPY services/rust/client-service/build.rs ./client-service/build.rs
COPY services/rust/client-service/proto ./client-service/proto

RUN mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    mkdir -p common/src && \
    echo "" > common/src/lib.rs

RUN cargo build --release --manifest-path client-service/Cargo.toml

RUN rm -rf client-service/src common/src

COPY services/rust/common/src ./common/src
COPY services/rust/client-service/src ./client-service/src

RUN touch common/src/lib.rs && \
    touch client-service/src/main.rs && \
    cargo build --release --manifest-path client-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/client-service .

EXPOSE 8080

CMD ["./client-service"]
