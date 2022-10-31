ARG BINARY_NAME=app
ARG RUST_TARGET_AMD64=x86_64-unknown-linux-musl
ARG RUST_TARGET_ARM32=armv7-unknown-linux-gnueabihf
ARG RUST_TARGET_ARM64=aarch64-unknown-linux-gnu
ARG RUNBASE_VERSION=3.16.2

# -----------------------------------------------------------------------
# ------------      BUILDER BASE    -------------------------------------
# -----------------------------------------------------------------------
FROM rust:1.64.0 as base
RUN apt update && \
    apt install -y pkg-config libssl-dev && \
    mkdir -p /app
ENV PKG_CONFIG_SYSROOT_DIR=/
WORKDIR /app

# -----------------------------------------------------------------------
# ------------     PLATFORM BUILDERS     --------------------------------
# -----------------------------------------------------------------------

# -->> AMD 64 <<--
FROM base as base-amd64
ARG RUST_TARGET_AMD64
ENV RUST_TARGET=$RUST_TARGET_AMD64
RUN rustup target add $RUST_TARGET &&\
    rustup toolchain install stable-$RUST_TARGET &&\
    apt install -y \
    # musl compiler
    musl-tools musl-dev \
    # cc linker fix -> https://stackoverflow.com/questions/66556519/cannot-compile-rust-project-linking-with-cc-failed
    gcc-multilib
ENV LDFLAGS=-lpthread
WORKDIR /app

# -->> ARM 64 <<--
# Inspired by https://github.com/skerkour/black-hat-rust/blob/main/ch_12/rat/docker/Dockerfile.aarch64
FROM base as base-arm64
ARG RUST_TARGET_ARM64
ENV RUST_TARGET=$RUST_TARGET_ARM64
RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross && \
    rustup target add $RUST_TARGET &&\
    rustup toolchain install stable-$RUST_TARGET
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# -->> ARM 32 <<--
FROM base as base-arm32
ARG RUST_TARGET_ARM32
ENV RUST_TARGET=$RUST_TARGET_ARM32
RUN apt install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross && \
    rustup target add $RUST_TARGET && \
    rustup toolchain install stable-$RUST_TARGET
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
    CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++


# -----------------------------------------------------------------------
# ------------      FINAL BUILDER   -------------------------------------
# -----------------------------------------------------------------------
#                /   Amd64 builder   \
#   Base builder -   Arm64 builder   - Final Builder
#                \   Arm32 builder   /
FROM base-$TARGETARCH AS final-builder

COPY . .

RUN cargo install --release --target=$RUST_TARGET


# -----------------------------------------------------------------------
# ------------        RUNNER        -------------------------------------
# -----------------------------------------------------------------------
FROM alpine:$RUNBASE_VERSION
#Runner with ssl support

# Declare args in the runner scope to be able to use it
ARG BINARY_NAME
LABEL authors="red.avtovo@gmail.com"

COPY --from=final-builder /usr/local/cargo/bin/${BINARY_NAME} /opt/

ENV RUST_LOG="info"
ENV BINARY_NAME=${BINARY_NAME}

CMD /opt/${BINARY_NAME}
