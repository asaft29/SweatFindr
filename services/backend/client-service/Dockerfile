FROM rust:1.90.0-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY services/backend/ .
RUN cargo chef prepare --recipe-path recipe.json --bin client-service

FROM chef AS builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY services/backend/ .
RUN cargo build --release --bin client-service

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
