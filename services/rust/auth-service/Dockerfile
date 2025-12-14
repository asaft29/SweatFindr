FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/rust/Cargo.toml ./Cargo.toml
COPY services/rust/Cargo.lock ./Cargo.lock

COPY services/rust/auth-service/Cargo.toml ./auth-service/Cargo.toml
COPY services/rust/client-service/Cargo.toml ./client-service/Cargo.toml
COPY services/rust/event-service/Cargo.toml ./event-service/Cargo.toml
COPY services/rust/email-service/Cargo.toml ./email-service/Cargo.toml
COPY services/rust/common/Cargo.toml ./common/Cargo.toml

RUN mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p email-service/src && \
    echo "fn main() {}" > email-service/src/main.rs && \
    mkdir -p common/src && \
    echo "" > common/src/lib.rs

COPY services/rust/auth-service/proto ./auth-service/proto
COPY services/rust/auth-service/build.rs ./auth-service/build.rs

RUN cargo build --release --manifest-path auth-service/Cargo.toml

RUN rm -rf auth-service/src common/src

COPY services/rust/common/src ./common/src
COPY services/rust/auth-service/src ./auth-service/src

RUN touch common/src/lib.rs && \
    touch auth-service/src/main.rs && \
    cargo build --release --manifest-path auth-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

COPY --from=builder /app/target/release/auth-service .

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 50051

CMD ["./auth-service"]
