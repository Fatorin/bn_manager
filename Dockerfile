# build
FROM rust:1.83 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools

WORKDIR /app

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

# stage
FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bn_manager /app/bn_manager

COPY static /app/static
COPY template /app/template
COPY migrations /app/migrations
COPY i18n /app/i18n

CMD ["/app/bn_manager"]

EXPOSE 3000
