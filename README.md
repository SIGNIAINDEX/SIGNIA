![1500x500](https://github.com/user-attachments/assets/43f1a906-29ba-409c-999a-3df4e156a60f)

# SIGNIA

**SIGNIA is coming to Solana.**  
A structure-level on-chain system for compiling real-world structures into verifiable forms.

**CA: 2rWMKqxBwMbgjc5vKa1U72Mt4ut9Pfcm7oZbHGaxpump**

<!-- ====== Primary Links (Green Theme Buttons) ====== -->
<p align="center">
  <a href="https://signialab.org/">
    <img alt="Website" src="https://img.shields.io/badge/Website-signialab.org-00C853?style=for-the-badge&logo=googlechrome&logoColor=white">
  </a>
  <a href="https://x.com/SIGNIAINDEX">
    <img alt="X (Twitter)" src="https://img.shields.io/badge/X-@SIGNIAINDEX-00C853?style=for-the-badge&logo=x&logoColor=white">
  </a>
  <a href="https://x.com/i/communities/2012892721191551236">
    <img alt="X Community" src="https://img.shields.io/badge/Community-Join-00C853?style=for-the-badge&logo=x&logoColor=white">
  </a>
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/License-Apache--2.0-00C853?style=flat-square">
  <img alt="Language" src="https://img.shields.io/badge/Rust-Workspace-00C853?style=flat-square">
  <img alt="Node" src="https://img.shields.io/badge/Node-20+-00C853?style=flat-square">
  <img alt="Status" src="https://img.shields.io/badge/Status-Active%20Development-00C853?style=flat-square">
</p>

---

## What is SIGNIA?

SIGNIA is a **structure compiler**: it reads an existing real-world structure (a Git repository, dataset, spec, API schema, workflow, or config),
derives a typed model, then emits **canonical artifacts**:

- **Schema**: the normalized structure model (what the thing *is*)
- **Manifest**: deterministic inputs, versions, and build metadata (what was used)
- **Proof**: hashes and Merkle roots that make the output *verifiable*

The goal is not to copy execution logic on-chain. The goal is to make **structure** composable and auditable for on-chain systems.

---

## Why this exists

Most “bring Web2 to chain” attempts focus on content or execution. SIGNIA focuses on **structure**.

That unlocks:

- **Verifiable structure registries** (publish schema hashes on Solana)
- **Composable on-chain tooling** that can *understand* off-chain objects
- **Deterministic artifact pipelines** for reproducible builds
- **Ecosystem plugins** that turn different inputs into the same canonical IR

---

## Core properties

- **Deterministic**: same input → same bytes (canonical JSON, normalized paths/text)
- **Typed IR**: plugins produce a stable intermediate representation
- **Verifiable outputs**: stable hashing + Merkle proofs
- **Pluggable**: new input types can be added as sandboxed plugins
- **On-chain registry**: publish and resolve schema versions on Solana

---

## Repository layout

```
crates/
  signia-core/            # canonical IR, pipeline, determinism, hashing, proofs
  signia-plugins/         # plugin runtime + built-in plugins (repo/dataset/spec/api/workflow/config)
  signia-store/           # local persistence (sqlite) + object store (fs/s3) + proof verification
  signia-api/             # HTTP API (compile/verify/artifacts/plugins/registry)
  signia-cli/             # CLI (compile/verify/publish/fetch/plugins/doctor)
  signia-solana-client/   # Rust client helpers for the on-chain registry

programs/
  signia-registry/        # Anchor program: on-chain schema registry

sdk/
  ts/                     # TypeScript SDK for API + registry helpers
  python/                 # Python SDK for API + utils

console/
  web/                    # Next.js web console (compile/verify/registry/interface)
  interface/              # "Interface" service (project assistant over docs/schemas/examples)

schemas/
  v1/                     # JSON schemas for schema/manifest/proof

examples/                 # end-to-end examples with expected outputs
tests/                    # integration + e2e tests
scripts/                  # bootstrap/build/lint/test helpers
infra/                    # docker/k8s/terraform
.devcontainer/            # devcontainer config
```

---

## Quickstart (local)

### Prerequisites

- Rust (stable), Cargo
- Node.js **20+**
- pnpm **9+**
- Docker (optional, recommended for quick boot)

### 1) Bootstrap

```bash
./scripts/bootstrap.sh
```

### 2) Build everything

```bash
./scripts/build_all.sh
```

### 3) Run tests

```bash
./scripts/test_all.sh
```

---

## Running the stack (Docker)

```bash
docker compose up -d --build
```

- API: `http://localhost:8080`
- Console: `http://localhost:3000`
- Interface service: `http://localhost:8090`

To stop:

```bash
docker compose down -v
```

---

## Using the CLI

### Compile an input

```bash
# Example: compile a local file (dataset, spec, workflow, config, etc.)
./target/release/signia compile ./examples/dataset/sample.csv --out ./out
```

### Verify outputs

```bash
./target/release/signia verify ./out/schema.json ./out/proof.json
```

### Publish to Solana devnet

```bash
./target/release/signia publish --devnet ./out/manifest.json
```

> Note: publishing requires a Solana keypair and configured RPC URL.

---

## Using the API

Start `signia-api` (local example):

```bash
cargo run -p signia-api
```

Then:

- `POST /v1/compile` → returns schema/manifest/proof
- `POST /v1/verify` → verifies schema + proof
- `GET /v1/artifacts/{hash}` → fetch by content hash
- `GET /v1/plugins` → list supported types + versions
- `POST /v1/registry/publish` → publish schema hash to Solana
- `GET /v1/registry/resolve` → resolve latest/version

OpenAPI spec: `docs/api/openapi.yaml`

---

## On-chain registry (Anchor)

The Solana program maintains a registry of published schema hashes with version pointers.

- Program: `programs/signia-registry/`
- Tests: `programs/signia-registry/tests/*.ts`

Typical flow:

1. Initialize registry
2. Register a schema (hash + metadata)
3. Publish a version pointer
4. Resolve latest for a given namespace/type

---

## SDKs

### TypeScript SDK

```bash
cd sdk/ts
pnpm install
pnpm build
```

Use it to:
- call the SIGNIA HTTP API
- compute canonical hashes (client-side)
- interact with the registry program

### Python SDK

```bash
cd sdk/python
python -m pip install -e .
```

Use it to:
- call the SIGNIA HTTP API
- verify proofs locally
- canonicalize JSON and compute hashes

---

## Console

The web console is a Next.js app used for:

- compiling inputs via the API
- viewing schema/manifest/proof
- verifying outputs
- browsing published registry entries

```bash
cd console/web
pnpm install
pnpm dev
```

---

## Interface (Project Assistant)

The interface service powers an on-site assistant trained on:

- docs
- schemas
- examples
- frequently asked questions

It provides a project-centric Q&A and deployment guidance layer, designed for embedding into the website console.

```bash
cd console/interface
pnpm install
pnpm dev
```

---

## Determinism & verifiability

SIGNIA is built around reproducibility:

- Canonical JSON serialization (stable key ordering)
- Path normalization
- Text normalization
- Stable hashing (BLAKE3)
- Merkle proof construction & verification

See:
- `docs/determinism/*`
- `docs/schemas/*`

---

## Security

- Vulnerability reporting: see `SECURITY.md`
- Threat model: `docs/security/security/threat-model.md`
- Supply-chain hardening: `docs/security/security/supply-chain.md`

---

## Contributing

See `CONTRIBUTING.md`.

---

## License

Apache-2.0. See `LICENSE`.

---

## Disclaimer

SIGNIA is under active development. APIs, schemas, and on-chain interfaces may change before a stable release.
