FROM rust:1.49 as build
WORKDIR /usr/src/app
COPY src src
COPY Cargo.lock .
COPY Cargo.toml .
ENV DATABASE_URL="mysql://root:secret@host.docker.internal:3306/serlo"
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/src/app/target/release/database-layer-actix .
CMD ["./database-layer-actix"]
EXPOSE 8080
