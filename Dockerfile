# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS node

FROM rust:1.88-bookworm AS builder

COPY --from=node /usr/local /usr/local

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY frontend/package.json frontend/pnpm-lock.yaml ./frontend/
COPY frontend/patches ./frontend/patches
RUN npm install -g pnpm
RUN pnpm --dir frontend install --frozen-lockfile

COPY src ./src
COPY frontend ./frontend
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ai-guard /usr/local/bin/ai-guard

ENV AI_GUARD_BIND=0.0.0.0:8787 \
    AI_GUARD_DATA_DIR=/data

VOLUME ["/data"]
EXPOSE 8787

ENTRYPOINT ["ai-guard"]
