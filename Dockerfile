# All respects goes to the author of https://shaneutt.com/blog/rust-fast-small-docker-image-builds/
FROM ethereum/solc:nightly-alpine as solidity-compiler

WORKDIR /root/

COPY ./src/contract/* ./

RUN solc --overwrite --abi --bin priceoracle.sol -o .

FROM rust:latest as cargo-build

RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/priceoracle

COPY . .

COPY --from=solidity-compiler /root/PriceOracle.* ./src/contract/

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