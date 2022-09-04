ARG BINARY_NAME=app
# https://hub.docker.com/_/rust/tags
ARG RUST_VERSION=1.67.0
# https://hub.docker.com/_/ubuntu/tags
ARG RUNBASE_UBUNTU_VERSION=23.04

# -----------------------------------------------------------------------
# ------------      BUILDER BASE    -------------------------------------
# -----------------------------------------------------------------------
FROM rust:$RUST_VERSION as base
RUN apt-get update && \
    apt-get install -y \
    # openssl
    libssl-dev \
    # required for compilation of diesel
    libpq-dev && \
    mkdir -p /app
ENV PKG_CONFIG_SYSROOT_DIR=/
WORKDIR /app

# -----------------------------------------------------------------------
# ------------          BUILDER       -------------------------------------
# -----------------------------------------------------------------------
FROM base AS builder
ARG TARGETARCH
ARG TARGETVARIANT

COPY . .

RUN echo "Building '$TARGETARCH$TARGETVARIANT' ..." &&\
    case "$TARGETARCH$TARGETVARIANT" in \
    "amd64" | "linux/amd64") \
      export RUST_TARGET=x86_64-unknown-linux-gnu \
      ;; \
    "arm64" | "linux/arm64/v8") \
      export RUST_TARGET=aarch64-unknown-linux-gnu \
             CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
             CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
             CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++; \
      apt-get install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross \
      ;; \
    "arm32v7"| "linux/arm/v7")  \
      export RUST_TARGET=armv7-unknown-linux-gnueabihf  \
             CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
             CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
             CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++; \
      apt-get install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross \
      ;; \
    *) \
      echo "Error: unsupported target!"; \
      exit 1 \
      ;; \
    esac &&\
    echo "RUST_TARGET=$RUST_TARGET" &&\
    rustup target add $RUST_TARGET && \
    cargo install --target=$RUST_TARGET --path .

# -----------------------------------------------------------------------
# ------------        RUNNER        -------------------------------------
# -----------------------------------------------------------------------
FROM ubuntu:$RUNBASE_UBUNTU_VERSION
#Runner with ssl support

# Declare args in the runner scope to be able to use it
ARG BINARY_NAME
LABEL authors="red.avtovo@gmail.com"

RUN apt-get update && \
    apt-get install -y \
    # Postgres client
    libpq5 && \
    apt-get clean

COPY --from=builder /usr/local/cargo/bin/${BINARY_NAME} /opt/

ENV RUST_LOG="info"
ENV BINARY_NAME=${BINARY_NAME}

CMD /opt/${BINARY_NAME}
