# NOTE: could use rustlang/rust:nightly + lld, but mold linker only ships binaries for Debian 12 (bookworm) which isn't what nightly uses.
# NOTE: unfortunately, mold does not support wasm https://github.com/rui314/mold/issues/555
# FROM rust:1.67-bookworm
FROM rustlang/rust:nightly

RUN apt-get update && \
  apt-get install -y libasound2-dev libudev-dev lld clang && \
  apt-get clean && rm -rf /var/lib/apt/lists/*

# NOTE: switch to nightly if base image is rust:1.67-bookworm
# RUN rustup toolchain install nightly
# RUN rustup default nightly
RUN rustup target install wasm32-unknown-unknown

RUN cargo install trunk wasm-pack
