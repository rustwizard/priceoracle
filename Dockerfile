# All respects goes to the author of https://shaneutt.com/blog/rust-fast-small-docker-image-builds/

FROM rust:latest as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/priceoracle

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/priceoracle*

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest

RUN addgroup -g 1000 priceoracle

RUN adduser -D -s /bin/sh -u 1000 -G priceoracle priceoracle

WORKDIR /home/priceoracle/bin/

COPY --from=cargo-build /usr/src/priceoracle/target/x86_64-unknown-linux-musl/release/priceoracle .

RUN chown priceoracle:priceoracle priceoracle

USER priceoracle

EXPOSE 8080

ENTRYPOINT ["./priceoracle"]