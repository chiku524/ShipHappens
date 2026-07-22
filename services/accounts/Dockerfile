FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
# Standalone package build (workspace lockfile lives at repo root).
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/pudgymon-accounts /usr/local/bin/pudgymon-accounts
COPY migrations ./migrations
ENV PORT=8787
EXPOSE 8787
CMD ["pudgymon-accounts"]
