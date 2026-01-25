# syntax=docker/dockerfile:1

FROM rust:1.93-bookworm AS builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY programs ./programs
# Some workspaces include console/sdk; keep copy minimal for Rust build
RUN cargo build -p signia-api --release || true

# Build
COPY . .
RUN cargo build -p signia-api --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /srv
COPY --from=builder /app/target/release/signia-api /usr/local/bin/signia-api
COPY infra/docker/runtime/entrypoint.sh /entrypoint.sh
COPY infra/docker/runtime/healthcheck.sh /healthcheck.sh
RUN chmod +x /entrypoint.sh /healthcheck.sh

ENV SIGNIA_BIND_ADDR=0.0.0.0:8787
EXPOSE 8787
HEALTHCHECK --interval=10s --timeout=3s --retries=10 CMD ["/healthcheck.sh","http://127.0.0.1:8787/healthz"]
ENTRYPOINT ["/entrypoint.sh"]
CMD ["signia-api"]
