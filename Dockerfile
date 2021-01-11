# https://github.com/sha-el/Vouch/blob/master/Dockerfile

FROM rust:1.49 AS builder
RUN apt-get update && apt-get -y install ca-certificates cmake musl-tools libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p /src \
        && echo "fn main() {println!(\"broken\")}" > /src/main.rs \
        && cargo build

COPY src src/
COPY .env /
RUN cargo build

FROM debian:10-slim
WORKDIR /app
COPY --from=builder /target/debug/magic /app/magic

CMD ["/app/magic"]
