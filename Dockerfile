FROM rust:latest as cargo-build

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/juno

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache dumb-init

WORKDIR /juno

COPY --from=cargo-build /usr/src/juno/target/x86_64-unknown-linux-musl/release/juno .

ENTRYPOINT ["/usr/bin/dumb-init", "--", "/juno/juno"]

# Use CMD TO Pass args

