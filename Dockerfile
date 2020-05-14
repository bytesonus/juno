FROM rust:latest as build

RUN rustup target add x86_64-unknown-linux-gnu

WORKDIR /usr/src/juno

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-gnu

FROM ubuntu:focal

WORKDIR /juno

COPY --from=build /usr/src/juno/target/x86_64-unknown-linux-gnu/release/juno .

CMD ["init", "--", "/juno/juno"]
