# SIGNIA Deployment Guide

This document describes **production-ready deployment flows** for SIGNIA, including:
- Local development
- Docker Compose (single-host)
- Kubernetes (multi-service)
- Terraform (cloud infrastructure scaffolding)
- Solana program deployment (Anchor)
- Operational checklists (secrets, migrations, upgrades, rollback)

> This guide assumes the repository layout described in `README.md`.
> All paths are relative to repo root unless stated otherwise.

---

## Table of contents

- [Architecture summary](#architecture-summary)
- [Services and ports](#services-and-ports)
- [Environment variables](#environment-variables)
- [Build matrix](#build-matrix)
- [Local deployment (no Docker)](#local-deployment-no-docker)
- [Docker Compose deployment](#docker-compose-deployment)
- [Kubernetes deployment](#kubernetes-deployment)
- [Terraform deployment](#terraform-deployment)
- [Solana program deployment (Anchor)](#solana-program-deployment-anchor)
- [Database migrations](#database-migrations)
- [Object store configuration](#object-store-configuration)
- [Auth & rate limiting](#auth--rate-limiting)
- [Observability](#observability)
- [Upgrade strategy](#upgrade-strategy)
- [Rollback strategy](#rollback-strategy)
- [Production checklist](#production-checklist)
- [Troubleshooting](#troubleshooting)

---

## Architecture summary

SIGNIA is deployed as a set of cooperating components:

- **signia-api** (Rust HTTP API): compile/verify/artifacts/plugins/registry routes.
- **signia-store** (Rust library): persistence + object store + caching used by the API.
- **signia-registry** (Solana program): on-chain registry of schema hashes and version pointers.
- **console/web** (Next.js): user-facing web console.
- **console/interface** (Node service): “Interface” assistant over docs/schemas/examples (optional).

A typical production deployment is:

```
Users -> Console (Next.js) -> signia-api -> store (sqlite + fs/s3)
                              |
                              +-> Solana RPC -> signia-registry program
```

---

## Services and ports

Default ports (can be changed via env):

- `signia-api`: `8080` (HTTP)
- `console/web`: `3000` (HTTP)
- `console/interface`: `8090` (HTTP)

Docker Compose uses the same defaults unless overridden.

---

## Environment variables

### signia-api

Common variables (names may also exist in `crates/signia-api/src/config.rs`):

- `SIGNIA_ENV`: `dev` | `staging` | `prod`
- `SIGNIA_LOG`: log filter (e.g. `info`, `debug`, `trace`)
- `SIGNIA_BIND_ADDR`: e.g. `0.0.0.0:8080`
- `SIGNIA_PUBLIC_BASE_URL`: e.g. `https://api.signialab.org`
- `SIGNIA_DB_URL`: e.g. `sqlite:///var/lib/signia/signia.db`
- `SIGNIA_OBJECT_STORE`: `fs` | `s3`
- `SIGNIA_OBJECT_ROOT`: e.g. `/var/lib/signia/objects` (FS mode)
- `SIGNIA_S3_BUCKET`: bucket name (S3 mode)
- `SIGNIA_S3_REGION`: e.g. `us-east-1`
- `SIGNIA_S3_ENDPOINT`: optional (S3-compatible providers)
- `SIGNIA_S3_ACCESS_KEY_ID`: secret
- `SIGNIA_S3_SECRET_ACCESS_KEY`: secret
- `SIGNIA_AUTH_MODE`: `none` | `bearer` | `hmac` (if implemented)
- `SIGNIA_AUTH_BEARER_TOKENS`: comma-separated list (for simple bearer mode)
- `SIGNIA_RATE_LIMIT_RPS`: requests per second per key/IP
- `SIGNIA_RATE_LIMIT_BURST`: burst capacity
- `SIGNIA_SOLANA_RPC_URL`: e.g. `https://api.devnet.solana.com`
- `SIGNIA_SOLANA_COMMITMENT`: `processed` | `confirmed` | `finalized`
- `SIGNIA_KEYPAIR_PATH`: server keypair for registry publishing (optional; not required for read-only)
- `SIGNIA_REGISTRY_PROGRAM_ID`: program id of `signia-registry` (devnet/mainnet)

### console/web (Next.js)

- `NEXT_PUBLIC_SIGNIA_API_URL`: e.g. `https://api.signialab.org`
- `NEXT_PUBLIC_SOLANA_RPC_URL`: optional (for client-side reads)
- `NEXT_PUBLIC_REGISTRY_PROGRAM_ID`: optional (for client-side registry reads)

### console/interface (Node)

- `SIGNIA_INTERFACE_BIND_ADDR`: e.g. `0.0.0.0:8090`
- `SIGNIA_INTERFACE_DOCS_ROOT`: e.g. `./docs`
- `SIGNIA_INTERFACE_SCHEMA_ROOT`: e.g. `./schemas`
- `SIGNIA_INTERFACE_EXAMPLES_ROOT`: e.g. `./examples`
- `SIGNIA_INTERFACE_MODEL_PROVIDER`: `openai` | `anthropic` | `local` (example)
- `SIGNIA_INTERFACE_MODEL_NAME`: model id/name
- `SIGNIA_INTERFACE_API_KEY`: secret token for model provider
- `SIGNIA_INTERFACE_STYLE`: `concise` | `technical` (example)

> Use `.env.example` as the base template, then populate secrets via your platform (K8s secrets, CI secrets, or secret manager).

---

## Build matrix

### Rust

- Build: `cargo build --release --locked`
- Workspace: can build specific crates (api/cli) for faster CI.

### Node / pnpm

- Install: `pnpm install --frozen-lockfile`
- Build:
  - `console/web`: `pnpm build`
  - `console/interface`: `pnpm build` (if present)
  - `sdk/ts`: `pnpm build` (optional for releases)

---

## Local deployment (no Docker)

### 1) Bootstrap

```bash
./scripts/bootstrap.sh
```

### 2) Build Rust artifacts

```bash
cargo build --release --locked
```

### 3) Run signia-api

```bash
export SIGNIA_ENV=dev
export SIGNIA_BIND_ADDR=0.0.0.0:8080
export SIGNIA_DB_URL=sqlite:///tmp/signia.db
export SIGNIA_OBJECT_STORE=fs
export SIGNIA_OBJECT_ROOT=/tmp/signia-objects
export SIGNIA_SOLANA_RPC_URL=https://api.devnet.solana.com

cargo run -p signia-api
```

### 4) Run console/web

```bash
cd console/web
pnpm install
export NEXT_PUBLIC_SIGNIA_API_URL=http://localhost:8080
pnpm dev
```

### 5) Optional: run interface service

```bash
cd console/interface
pnpm install
pnpm dev
```

Health checks:
- API: `GET /v1/health`
- Console: load `http://localhost:3000`

---

## Docker Compose deployment

### Files

- `docker-compose.yml`
- `infra/docker/api.Dockerfile`
- `infra/docker/console.Dockerfile`
- `infra/docker/interface.Dockerfile`
- `infra/docker/runtime/entrypoint.sh`
- `infra/docker/runtime/healthcheck.sh`

### 1) Configure environment

Create `.env` from `.env.example`:

```bash
cp .env.example .env
```

Fill values:
- database path / S3 credentials
- public API URL
- Solana RPC URL and program id

### 2) Build and start

```bash
docker compose up -d --build
```

### 3) Validate health

```bash
curl -fsS http://localhost:8080/v1/health
```

### 4) Logs

```bash
docker compose logs -f api
docker compose logs -f console
docker compose logs -f interface
```

### 5) Stop

```bash
docker compose down -v
```

---

## Kubernetes deployment

This repo includes sample manifests under `infra/k8s/`:

- `namespace.yaml`
- `api-deployment.yaml`
- `console-deployment.yaml`
- `interface-deployment.yaml`
- `ingress.yaml`

### 1) Create namespace

```bash
kubectl apply -f infra/k8s/namespace.yaml
```

### 2) Create secrets

Recommended: store secrets in a secret manager (AWS SM / GCP SM / Vault) and sync to K8s.

At minimum, create a secret for:
- S3 credentials (if used)
- model provider key (if interface enabled)
- API auth tokens (if enabled)

Example (replace values):

```bash
kubectl -n signia create secret generic signia-secrets   --from-literal=SIGNIA_S3_ACCESS_KEY_ID=REPLACE_ME   --from-literal=SIGNIA_S3_SECRET_ACCESS_KEY=REPLACE_ME   --from-literal=SIGNIA_INTERFACE_API_KEY=REPLACE_ME
```

### 3) Apply deployments

```bash
kubectl apply -f infra/k8s/api-deployment.yaml
kubectl apply -f infra/k8s/console-deployment.yaml
kubectl apply -f infra/k8s/interface-deployment.yaml
kubectl apply -f infra/k8s/ingress.yaml
```

### 4) Verify

```bash
kubectl -n signia get pods
kubectl -n signia get svc
kubectl -n signia describe deploy signia-api
```

### 5) Ingress/TLS

Update `infra/k8s/ingress.yaml` for:
- hostnames: `api.signialab.org`, `signialab.org`
- TLS secrets (cert-manager recommended)

---

## Terraform deployment

`infra/terraform/` provides a scaffold for:

- networking
- storage (object store + optional db)
- service deployment wiring

### Typical flow

1. Initialize:
   ```bash
   cd infra/terraform/environments/prod
   terraform init
   ```
2. Plan:
   ```bash
   terraform plan
   ```
3. Apply:
   ```bash
   terraform apply
   ```

**Important:** terraform modules in this repo are templates. Customize them to your provider:
- AWS: S3 + EKS + ALB Ingress
- GCP: GCS + GKE + HTTP LB
- DigitalOcean: Spaces + DOKS
- Fly.io / Render / Railway: use Docker deployment instead

---

## Solana program deployment (Anchor)

Program: `programs/signia-registry`

### 1) Install Solana + Anchor

- Solana CLI matching your target cluster
- Anchor (via `cargo install --git https://github.com/coral-xyz/anchor avm --locked` and `avm install latest`)

### 2) Configure cluster

Devnet:

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 2
```

### 3) Build program

```bash
cd programs/signia-registry
anchor build
```

### 4) Deploy

```bash
anchor deploy --provider.cluster devnet
```

Record:
- program id
- IDL output (Anchor generates idl artifacts)

### 5) Run program tests

```bash
anchor test
```

### 6) Configure API + Console

Set:
- `SIGNIA_REGISTRY_PROGRAM_ID`
- `SIGNIA_SOLANA_RPC_URL`

---

## Database migrations

`signia-store` uses SQLite by default and includes migrations:

- `crates/signia-store/src/kv/migrations/*.sql`

Recommended approach:
- Run migrations at startup (idempotent)
- Keep schema changes backwards compatible when possible
- For breaking changes, plan a maintenance window or dual-write strategy

If your `signia-store` implementation supports an explicit migration command, run it during deploy before traffic cutover.

---

## Object store configuration

### FS mode (single host)

- `SIGNIA_OBJECT_STORE=fs`
- `SIGNIA_OBJECT_ROOT=/var/lib/signia/objects`

Ensure:
- the directory exists
- correct permissions for the runtime user
- backup strategy (rsync/snapshots)

### S3 mode (multi-host)

- `SIGNIA_OBJECT_STORE=s3`
- `SIGNIA_S3_BUCKET=...`
- `SIGNIA_S3_REGION=...`
- credentials in secrets

Ensure:
- bucket versioning enabled (recommended)
- SSE encryption enabled (recommended)
- lifecycle rules for cold storage (optional)

---

## Auth & rate limiting

### Auth

In production, do not leave the API open unless intended.

Common patterns:
- simple bearer token (internal tools)
- HMAC signature per request (integrations)
- gateway auth (Cloudflare / API Gateway / Ingress auth)

Set:
- `SIGNIA_AUTH_MODE=bearer`
- `SIGNIA_AUTH_BEARER_TOKENS=...`

### Rate limiting

Set:
- `SIGNIA_RATE_LIMIT_RPS`
- `SIGNIA_RATE_LIMIT_BURST`

Also consider upstream rate limiting at the edge (Ingress / CDN / WAF).

---

## Observability

### Logging

Set:
- `SIGNIA_LOG=info` (prod default)
- `SIGNIA_LOG=debug` (short-term debugging)

### Metrics and tracing

If supported by `crates/signia-api/src/telemetry.rs`:
- OpenTelemetry export to Tempo/Jaeger
- Prometheus metrics endpoint

In K8s, expose metrics as a Service and scrape via Prometheus Operator.

### Health checks

- API: `/v1/health`
- Docker: use `infra/docker/runtime/healthcheck.sh`
- K8s: use readiness/liveness probes

---

## Upgrade strategy

### API

- Prefer rolling updates (K8s Deployment)
- Keep API responses backwards compatible when possible
- Version all endpoints under `/v1/*` and introduce `/v2/*` for breaking changes

### Store and migrations

- Migrations must be forward compatible with old binaries during a rolling deploy
- If not possible, do a two-phase deploy (maintenance window)

### Console

- Can be deployed independently as long as it points to a compatible API version

### Interface

- Optional component; deploy independently
- Keep retriever/index formats versioned

---

## Rollback strategy

- Keep the previous container image available
- Keep `VERSION` tags for releases
- For DB migrations, avoid irreversible migrations without a rollback plan
- For on-chain changes, prefer version pointers rather than hard deletes

If an upgrade fails:
1. Roll back API/console images
2. Restore DB snapshot (if needed)
3. Repoint DNS/Ingress to stable versions

---

## Production checklist

### Security

- [ ] No secrets committed (scan repo; rotate keys)
- [ ] CI permissions minimized (workflows reviewed)
- [ ] Dependabot enabled
- [ ] CodeQL enabled (or explicitly disabled)
- [ ] API auth enabled (unless intentionally public)
- [ ] Rate limiting enabled at API and edge
- [ ] CORS restricted to the console domain(s)

### Reliability

- [ ] Health checks passing
- [ ] Logs shipped to a central system
- [ ] Backups configured (db + object store)
- [ ] Resource limits set (CPU/memory)
- [ ] Alerting on error rates and latency

### Solana

- [ ] Program id recorded and monitored
- [ ] RPC provider configured (rate limits, failover)
- [ ] Commitment level chosen
- [ ] Key management strategy for publishing (HSM/secret manager)

---

## Troubleshooting

### API won't start

- Verify DB path is writable
- Verify object store credentials / directory permissions
- Verify env vars are present
- Check logs: `SIGNIA_LOG=debug`

### Console cannot reach API

- Ensure `NEXT_PUBLIC_SIGNIA_API_URL` is correct
- Verify CORS in API and Ingress
- Verify API is reachable from the browser network

### CodeQL / CI failures

- Ensure pnpm is installed in workflows (corepack/pnpm-action)
- Ensure Rust builds are pinned and use `--locked`
- Verify `permissions.security-events: write` for CodeQL

### Registry publishing fails

- Verify `SIGNIA_KEYPAIR_PATH`
- Verify RPC URL and cluster config
- Verify program id is correct for the cluster

---

## Notes

- This repository includes Docker/K8s/Terraform scaffolding under `infra/`.
- For a minimal production deployment, Docker Compose is the simplest stable baseline.
- For a scalable deployment, use K8s with an external object store (S3) and centralized logging.
