FROM ethereum/solc:stable-alpine as solidity-compiler

WORKDIR /root/

COPY ./src/contract/* ./

RUN solc --overwrite --abi --bin priceoracle.sol -o .

FROM rust:slim-stretch as cargo-build

RUN apt-get update && apt-get -y install libssl-dev pkg-config ca-certificates

WORKDIR /usr/src/priceoracle

COPY . .

COPY --from=solidity-compiler /root/PriceOracle.* ./src/contract/

RUN cargo build --release

FROM debian:stretch-slim

RUN apt-get update && apt-get -y install libssl-dev openssl ca-certificates

WORKDIR /home/priceoracle/bin/

COPY --from=cargo-build /usr/src/priceoracle/target/release/priceoracle .

EXPOSE 8080

ENTRYPOINT ["./priceoracle"]