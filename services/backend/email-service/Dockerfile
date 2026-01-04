FROM rust:1.90.0-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY services/backend/ .

RUN mkdir -p email-service/src && \
    echo "fn main() {}" > email-service/src/main.rs && \
    mkdir -p client-service/src && \
    echo "fn main() {}" > client-service/src/main.rs && \
    mkdir -p event-service/src && \
    echo "fn main() {}" > event-service/src/main.rs && \
    mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs

RUN cargo build --release --manifest-path email-service/Cargo.toml

RUN rm -rf email-service/src

COPY services/backend/email-service/src ./email-service/src

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
