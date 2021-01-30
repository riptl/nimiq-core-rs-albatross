# syntax = docker/dockerfile:1.0-experimental

# Builds a light Alpine image from sources.
# Requires Docker buildkit.

ARG RUST_IMAGE=rustlang/rust:nightly-slim
FROM $RUST_IMAGE AS builder

# Fetch dependencies.
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Copy sources.
ADD . /build
WORKDIR /build

# Build.
RUN \
  --mount=type=cache,target=/build/target \
  --mount=type=cache,target=/root/.cargo \
  cargo build --bin nimiq-client && mv /build/target/debug/nimiq-client /build/

# Light stage.
FROM ubuntu:20.04

# Install dependencies.
RUN apt-get update \
 && apt-get install -y libssl1.1 \
 && apt-get clean

# Run as unprivileged user.
RUN adduser --disabled-password --home /home/nimiq --shell /bin/bash --uid 1001 nimiq
USER nimiq

# Pull image from builder.
COPY --chown=root:root --from=builder /build/nimiq-client /usr/local/bin/nimiq-client
ENTRYPOINT ["/usr/local/bin/nimiq-client"]

# https://github.com/opencontainers/image-spec/blob/master/annotations.md
LABEL \
  org.opencontainers.image.title="Nimiq core-rs-albatross" \
  org.opencontainers.image.description="Rust implementation of the Nimiq Blockchain Core Albatross Branch (Buildkit Ubuntu image)" \
  org.opencontainers.image.url="https://github.com/nimiq/core-rs-albatross" \
  org.opencontainers.image.vendor="Nimiq Foundation" \
  org.opencontainers.image.licenses="Apache-2.0"
