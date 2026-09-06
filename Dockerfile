FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static perl

WORKDIR /app

COPY . .

ARG VERSION=dev
RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/Aperio /app/aperio

ARG VERSION=dev
ENV APERIO_VERSION=${VERSION}
EXPOSE 3000

ENTRYPOINT ["/app/aperio"]
