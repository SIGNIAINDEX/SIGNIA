# Contributing to SIGNIA

Thanks for your interest in contributing. This document explains how to set up the repo, propose changes, and land PRs safely.

---

## Table of contents

- [Code of Conduct](#code-of-conduct)
- [What you can contribute](#what-you-can-contribute)
- [Repo overview](#repo-overview)
- [Development prerequisites](#development-prerequisites)
- [Local setup](#local-setup)
- [Common workflows](#common-workflows)
- [Making a change](#making-a-change)
- [Commit and PR conventions](#commit-and-pr-conventions)
- [Testing](#testing)
- [Security issues](#security-issues)
- [Releases](#releases)
- [Project governance](#project-governance)
- [License](#license)

---

## Code of Conduct

This project is governed by the Code of Conduct in [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md). By participating, you agree to abide by it.

---

## What you can contribute

We accept contributions across the whole system:

- **Core**: canonical IR, determinism, hashing, proofs
- **Plugins**: new input types, better inference, improved normalization
- **Store**: persistence, object layouts, caching, verification
- **API**: HTTP endpoints, auth/rate limiting, OpenAPI docs
- **CLI**: UX, output formatting, additional commands and recipes
- **Registry program**: Anchor program improvements and audits
- **SDKs**: TS/Python client improvements and docs
- **Console + Interface**: UI, DX, and the assistant retrieval layer
- **Docs**: architecture, threat model, determinism contract, examples

If you are unsure where a feature belongs, open a discussion or a “question” issue.

---

## Repo overview

Top-level layout:

- `crates/` — Rust workspace crates:
  - `signia-core`, `signia-plugins`, `signia-store`, `signia-api`, `signia-cli`, `signia-solana-client`
- `programs/` — Solana programs (Anchor)
- `sdk/` — TypeScript + Python SDKs
- `console/` — Next.js console + the “Interface” service
- `schemas/` — versioned JSON schemas (schema/manifest/proof)
- `docs/` — specifications, determinism contract, security docs
- `examples/` — end-to-end runnable examples
- `tests/` — integration and e2e tests
- `scripts/` — bootstrap/build/lint/test helpers
- `infra/` — Docker/K8s/Terraform deployment scaffolding

---

## Development prerequisites

### Required

- **Rust** (stable) + `cargo`
- **Node.js** 20+
- **pnpm** 9+
- Git

### Optional (recommended)

- Docker + Docker Compose
- Solana CLI (for devnet/localnet publishing)
- Anchor (for program development and tests)

---

## Local setup

### 1) Clone

```bash
git clone <YOUR_FORK_URL>
cd signia
```

### 2) Bootstrap toolchains

```bash
./scripts/bootstrap.sh
```

This script is designed to:
- install or validate Rust toolchain + components (fmt/clippy)
- ensure Node and pnpm are present
- install JS dependencies in `console/` and `sdk/ts/` where relevant

### 3) Build

```bash
./scripts/build_all.sh
```

### 4) Run tests

```bash
./scripts/test_all.sh
```

If you prefer Make:

```bash
make bootstrap
make build
make test
```

---

## Common workflows

### Lint everything

```bash
./scripts/lint_all.sh
```

### Format code

Rust:

```bash
cargo fmt --all
```

TypeScript:

```bash
pnpm -r format
```

### Run the API locally

```bash
cargo run -p signia-api
```

Defaults to `http://localhost:8080`.

### Run the console locally

```bash
cd console/web
pnpm dev
```

Console defaults to `http://localhost:3000`.

### Run Interface service locally

```bash
cd console/interface
pnpm dev
```

Interface defaults to `http://localhost:8090`.

### Run via Docker Compose

```bash
docker compose up -d --build
```

---

## Making a change

### 1) Create a branch

Use a descriptive branch name:

- `feat/<short-description>`
- `fix/<short-description>`
- `docs/<short-description>`
- `chore/<short-description>`

Example:

```bash
git checkout -b feat/new-plugin-openrpc
```

### 2) Keep changes scoped

Prefer small PRs with a single theme. Large changes should be split or preceded by a design discussion.

### 3) Add tests

- Rust crates: unit tests + integration tests where needed
- API: request/response tests where possible
- Console/Interface: smoke tests and basic coverage if feasible

### 4) Update docs

If you change schemas, determinism behavior, CLI commands, or API surfaces, update:
- `docs/`
- `schemas/`
- `docs/api/openapi.yaml`

---

## Commit and PR conventions

### Commit messages

We recommend a conventional style:

- `feat: ...`
- `fix: ...`
- `docs: ...`
- `chore: ...`
- `refactor: ...`
- `test: ...`

Examples:

- `feat: add dataset schema inference for JSONL`
- `fix: stabilize canonical JSON ordering for floats`
- `docs: expand threat model for plugin sandbox`

### PR title and description

PRs should include:

- what changed and why
- how to test
- impact on compatibility
- links to issues/discussions

If relevant, include:
- screenshots (console changes)
- schema diffs or examples
- benchmark numbers (determinism/hashing performance)

---

## Testing

### Rust

Run workspace tests:

```bash
cargo test --workspace
```

Run a single crate:

```bash
cargo test -p signia-core
```

### API integration tests

From repo root:

```bash
cargo test --test api_compile_flow
```

### TypeScript

From repo root:

```bash
pnpm -r test
```

### E2E (optional)

Some tests are gated by env vars (Docker/devnet). See `tests/README` (if present) and the test files in `tests/e2e/`.

---

## Security issues

Do not file public issues for security-sensitive findings.

Follow [`SECURITY.md`](./SECURITY.md).

---

## Releases

Releases are cut by maintainers using the release workflow.

General process:

1. Update `VERSION` and `CHANGELOG.md`
2. Tag a release `vX.Y.Z`
3. GitHub Actions builds artifacts and publishes checksums
4. On-chain program deployments (if any) are coordinated separately

---

## Project governance

See [`GOVERNANCE.md`](./GOVERNANCE.md).

---

## License

By contributing, you agree that your contributions will be licensed under the project license (Apache-2.0). See [`LICENSE`](./LICENSE).

