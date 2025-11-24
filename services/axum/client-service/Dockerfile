FROM rust:1.90.0-alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

COPY services/axum/Cargo.toml ./Cargo.toml
COPY services/axum/Cargo.lock ./Cargo.lock

COPY services/axum/client-service/Cargo.toml ./client-service/Cargo.toml
COPY services/axum/event-service/Cargo.toml ./event-service/Cargo.toml
COPY services/axum/auth-service/Cargo.toml ./auth-service/Cargo.toml
COPY services/axum/common/Cargo.toml ./common/Cargo.toml

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

COPY services/axum/common/src ./common/src
COPY services/axum/client-service/src ./client-service/src

RUN touch common/src/lib.rs && \
    touch client-service/src/main.rs && \
    cargo build --release --manifest-path client-service/Cargo.toml

FROM alpine:latest

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /app/target/release/client-service .

EXPOSE 8080

CMD ["./client-service"]
