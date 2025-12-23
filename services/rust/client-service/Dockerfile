FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/rust/ .

RUN mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    mkdir -p email-service/src && \
    echo "fn main() {}" > email-service/src/main.rs

RUN cargo build --release --manifest-path client-service/Cargo.toml

RUN rm -rf client-service/src

COPY services/rust/client-service/src ./client-service/src

RUN touch client-service/src/main.rs && \
    cargo build --release --manifest-path client-service/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

COPY --from=builder /app/target/release/client-service .

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["./client-service"]
