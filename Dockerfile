FROM rust:1.78-alpine3.18 as build
WORKDIR /usr/src/app
RUN apk update && apk add --no-cache musl-dev pkgconfig openssl-dev
COPY server server
COPY test-utils test-utils
COPY Cargo.lock .
COPY Cargo.toml .
COPY .sqlx .sqlx
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release

FROM alpine:3.20
WORKDIR /usr/src/app
RUN apk update && apk add --no-cache openssl openssl-dev libgcc
COPY --from=build /usr/src/app/target/release/server .
CMD ["./server"]
EXPOSE 8080
