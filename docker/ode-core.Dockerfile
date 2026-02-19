FROM rust:1.75-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --package ode-core

FROM debian:bookworm-slim as runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /app

RUN useradd -m -u 1000 ode

COPY --from=builder /app/target/release/libode_core.so /app/libode_core.so
COPY --chown=ode:ode crates/ode-core/src ./src

USER ode

ENV LD_LIBRARY_PATH=/app:$LD_LIBRARY_PATH

CMD ["/bin/bash"]