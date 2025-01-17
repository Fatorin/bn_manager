# build
FROM rust:1.81 as builder

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -r src

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

# stage
FROM alpine:latest

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bn_manager /app/bn_manager

COPY static /app/static
COPY template /app/template

CMD ["/app/bn_manager"]

EXPOSE 3000
