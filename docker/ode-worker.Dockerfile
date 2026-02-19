FROM rust:1.75-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --package ode-worker

FROM debian:bookworm-slim as runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /app

RUN useradd -m -u 1000 ode

COPY --from=builder /app/target/release/ode-worker /app/ode-worker
COPY --chown=ode:ode crates/ode-worker/src ./src

USER ode

EXPOSE 9090

ENV RUST_LOG=info
ENV RUST_BACKTRACE=0

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

# Graceful shutdown
STOPSIGNAL SIGTERM

CMD ["/app/ode-worker"]