FROM rust:1.70 as build
WORKDIR /usr/src/app
COPY server server
COPY test-utils test-utils
COPY Cargo.lock .
COPY Cargo.toml .
COPY sqlx-data.json .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/src/app/target/release/server .
CMD ["./server"]
EXPOSE 8080
