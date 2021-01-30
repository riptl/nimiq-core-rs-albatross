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
   mkdir /build/artifacts \
&& cargo build \
     --bin nimiq-address \
     --bin nimiq-bls \
     --bin nimiq-client \
     --bin nimiq-rpc \
     --bin nimiq-signtx \
&& mv \
     /build/target/debug/nimiq-address \
     /build/target/debug/nimiq-bls \
     /build/target/debug/nimiq-client \
     /build/target/debug/nimiq-rpc \
     /build/target/debug/nimiq-signtx \
     /build/artifacts/

# Light stage.
FROM ubuntu:20.04

# Install dependencies.
RUN apt-get update \
 && apt-get install -y libssl1.1 \
 && apt-get clean

# Run as unprivileged user.
RUN adduser --disabled-password --home /home/nimiq --shell /bin/bash --uid 1001 nimiq
USER nimiq

# Install helper scripts.
COPY --chown=root:root ./scripts/docker_*.sh /usr/local/bin/

# Install artifacts from build stage.
COPY --chown=root:root --from=builder /build/artifacts/* /usr/local/bin/
CMD ["docker_run.sh"]

# https://github.com/opencontainers/image-spec/blob/master/annotations.md
LABEL \
  org.opencontainers.image.title="Nimiq core-rs-albatross" \
  org.opencontainers.image.description="Rust implementation of the Nimiq Blockchain Core Albatross Branch (Buildkit Ubuntu image)" \
  org.opencontainers.image.source="https://github.com/nimiq/core-rs-albatross" \
  org.opencontainers.image.vendor="Nimiq Foundation" \
  org.opencontainers.image.licenses="Apache-2.0"
