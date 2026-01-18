
# Overview

SIGNIA is a structure-level on-chain compilation system. It converts real-world, already-formed artifacts into **canonical, verifiable, and composable structural representations** that can be referenced on-chain (Solana) and consumed by off-chain systems.

SIGNIA is designed for determinism: identical inputs produce identical outputs, independent of environment.

---

## One-sentence definition

**SIGNIA compiles real-world structures into verifiable on-chain forms.**

---

## The problem SIGNIA solves

Modern systems publish “facts” in formats that are hard to reuse and hard to trust:

- Repositories change over time and do not provide immutable structural anchors.
- Specifications and API schemas are often referenced informally without strong verification.
- Workflows and configuration graphs drift across environments.
- Systems want to reference structure (dependencies, constraints, interfaces), not necessarily store or execute the content itself.

SIGNIA provides a minimal, deterministic layer that extracts **structure** and emits a canonical representation that can be independently verified and optionally registered on-chain.

---

## What counts as “structure”

In SIGNIA, structure means the stable relationships and constraints that describe a system:

- Entities (modules, packages, endpoints, tables, documents, datasets)
- Types (schemas, interfaces, fields, request/response shapes)
- Edges (dependencies, imports, references, ownership, version links)
- Constraints (required fields, invariants, compatibility boundaries)
- Metadata needed for verification (hashes, manifests, proofs)

SIGNIA intentionally separates structure from:

- Runtime behavior
- Execution semantics
- Hosting and distribution
- Human intent or interpretation

---

## What SIGNIA produces

For a supported input, SIGNIA produces a deterministic bundle:

- **Schema**: a canonical structure definition (versioned)
- **Manifest**: compilation metadata, inputs, dependencies, and output hashes
- **Proof**: verification material (e.g., Merkle root + inclusion proofs)

Optionally, the schema hash and minimal metadata can be published to a Solana registry program to make the structure addressable and discoverable on-chain.

---

## How it works (high level)

1. **Parse**: ingest an artifact (repo, spec, OpenAPI, workflow, dataset, config)
2. **Infer**: build a structural model (entities, edges, types, constraints)
3. **Canonicalize**: normalize ordering and encoding into deterministic form
4. **Compile**: emit schema + manifest + proof bundle
5. **Verify / Register**: verify locally; optionally register on-chain

---

## Determinism guarantees

SIGNIA aims to guarantee:

- Same input → same output
- Stable ordering rules for all collections
- Canonical JSON encoding for hashing and proofs
- Platform-independent path and newline normalization
- No wall-clock, locale, randomness, or environment-derived behavior

Determinism is enforced by:
- explicit canonicalization rules
- golden fixtures
- round-trip verification tests

---

## Trust and verification model

SIGNIA is designed to minimize trust assumptions:

- Any party can recompute the bundle from the same input and compare hashes.
- Proofs allow verifying integrity without needing to trust the compiler runner.
- On-chain registry records only minimal identifiers (hashes and metadata) and does not need to store full content.

SIGNIA does not claim that an input is “true” in the real world; it proves that a published structure corresponds to a specific, deterministic compilation of a specific input.

---

## What SIGNIA is not

SIGNIA is not:

- a smart contract runtime
- a deployment pipeline for application code
- a content storage network
- a blockchain indexer
- a general-purpose execution environment

SIGNIA does not execute application logic. It compiles structure.

---

## Components

### Compiler (off-chain)
Implements parsing, inference, canonicalization, compilation, and verification.

### Plugin system
Adapters that turn different input formats into the same structural intermediate representation.

### CLI
Developer-facing entrypoint: compile, verify, publish, fetch, inspect.

### API service
Automation interface for compilation and retrieval workflows.

### Registry program (Solana)
On-chain registry that stores schema identifiers (hashes), version links, and minimal metadata.

### SDKs + Console
Convenience integrations for application developers and operators.

---

## Typical use cases

- Verifiable publication of OpenAPI schemas and interface contracts
- Immutable anchoring of workflow graphs and configuration models
- Referenceable “structure snapshots” for governance and audit trails
- A composable structure layer for higher-level protocols and tooling

---

## Quickstart (conceptual)

- Compile a supported input into a bundle (schema/manifest/proof)
- Verify the bundle locally (independent check)
- Optionally register the schema hash on Solana
- Use the schema hash as a stable structural identifier

---

## Status

SIGNIA is under active development.

There is currently **no token issued**.

---

## Next

- Architecture: `docs/architecture.md`
- Determinism rules: `docs/data-model/determinism-rules.md`
- Schema specification: `docs/data-model/schema-v1.md`
- CLI reference: `docs/cli/usage.md`
- Registry program: `docs/onchain/registry-program.md`
