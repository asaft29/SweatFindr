FROM rust:1.90.0-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY services/backend/ .
RUN cargo chef prepare --recipe-path recipe.json --bin notification-service

FROM chef AS builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY services/backend/ .
RUN cargo build --release --bin notification-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/notification-service /usr/local/bin/notification-service

EXPOSE 8080

CMD ["notification-service"]
