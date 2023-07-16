FROM debian:12 as build-rs

RUN apt-get update
RUN apt-get install -y \
  curl \
  musl-tools

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/local/src/udp-mpegts

RUN mkdir -pv src/
RUN mkdir -pv examples/probe
RUN echo "fn main() {println!(\"---\")}" > ./src/main.rs
RUN echo "fn main() {println!(\"---\")}" > ./examples/probe/main.rs
COPY ./Cargo.toml ./
RUN RUSTFLAGS=-Clinker=musl-gcc cargo \
  build \
  --release \
  --verbose \
  --target=x86_64-unknown-linux-musl
RUN RUSTFLAGS=-Clinker=musl-gcc cargo \
  build \
  --release \
  --verbose \
  --example probe \
  --target=x86_64-unknown-linux-musl

RUN rm -rvf ./target/x86_64-unknown-linux-musl/debug/deps/va-ts*
RUN rm -rvf ./target/x86_64-unknown-linux-musl/debug/va-ts*
RUN rm -rvf ./target/x86_64-unknown-linux-musl/release/deps/va-ts*
RUN rm -rvf ./target/x86_64-unknown-linux-musl/release/va-ts*
RUN rm -rvf ./src/*
RUN rm -rvf ./examples/*

COPY ./src/ ./src/
COPY ./examples/ ./examples/
RUN RUSTFLAGS=-Clinker=musl-gcc cargo \
  build \
  --release \
  --verbose \
  --example probe \
  --target=x86_64-unknown-linux-musl

# RUN cargo test -- --nocapture

# -------------------------------------
FROM alpine:latest

COPY --from=build-rs /usr/local/src/udp-mpegts/target/x86_64-unknown-linux-musl/release/examples/probe /usr/local/bin/probe

RUN ls -lah /usr/local/bin/probe

ENV RUST_BACKTRACE=full

CMD ["probe", "--input", "udp://239.255.1.1:5500"]
