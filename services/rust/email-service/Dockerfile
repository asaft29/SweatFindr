FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/rust/Cargo.toml ./Cargo.toml
COPY services/rust/Cargo.lock ./Cargo.lock

COPY services/rust/email-service/Cargo.toml ./email-service/Cargo.toml
COPY services/rust/client-service/Cargo.toml ./client-service/Cargo.toml
COPY services/rust/event-service/Cargo.toml ./event-service/Cargo.toml
COPY services/rust/auth-service/Cargo.toml ./auth-service/Cargo.toml
COPY services/rust/common/Cargo.toml ./common/Cargo.toml

COPY services/rust/email-service/build.rs ./email-service/build.rs
COPY services/rust/email-service/proto ./email-service/proto

RUN mkdir -p email-service/src && \
    echo "fn main() {}" > email-service/src/main.rs && \
    mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    mkdir -p common/src && \
    echo "" > common/src/lib.rs

RUN cargo build --release --manifest-path email-service/Cargo.toml

RUN rm -rf email-service/src

COPY services/rust/email-service/src ./email-service/src

RUN touch email-service/src/main.rs && \
    cargo build --release --manifest-path email-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

COPY --from=builder /app/target/release/email-service .

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 50052

CMD ["./email-service"]
