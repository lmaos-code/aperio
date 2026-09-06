FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev pkgconf openssl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/Aperio /app/aperio

EXPOSE 3000

ENTRYPOINT ["/app/aperio"]
