
# SIGNIA Schemas

This directory contains the normative format definitions for SIGNIA artifacts:
- `schema.json` (graph schema format)
- `manifest.json` (bundle manifest format)
- `proof.json` (Merkle proof format)
- examples for common artifact kinds (repo, openapi, dataset, workflow)

These files are intended to be:
- machine-readable (JSON Schema)
- stable across versions
- used by the CLI, API service, and Console for validation and interoperability

If you are integrating SIGNIA into another system, you should start here.

---

## Directory structure

- `v1/`
  - `schema.json` — JSON Schema for SIGNIA Schema v1
  - `manifest.json` — JSON Schema for SIGNIA Manifest v1
  - `proof.json` — JSON Schema for SIGNIA Proof v1
  - `examples/`
    - `repo.schema.json` — example schema for repository structure
    - `openapi.schema.json` — example schema for OpenAPI structure
  - `dataset.schema.json` — example dataset schema (not a JSON Schema; an instance document)
  - `workflow.schema.json` — example workflow schema (not a JSON Schema; an instance document)

Notes:
- `schema.json`, `manifest.json`, and `proof.json` are JSON Schema definitions.
- Files ending with `*.schema.json` under `examples/` are instance examples, not JSON Schema definitions.

---

## Versioning policy

Schemas are versioned under `schemas/<version>/`.

Rules:
- `v1` is stable and should remain backwards compatible for parsers.
- breaking changes require a new directory (e.g., `v2/`).
- patch changes in a version should be strictly additive when possible.
- deprecations should be documented with clear migration guidance.

---

## Canonicalization and determinism

SIGNIA is built around deterministic outputs. Consumers should assume:
- JSON is canonicalized before hashing (key ordering, whitespace, floats)
- leaf keys in proofs are canonical and sorted
- identifiers (`ent:*`, `edge:*`) are stable and must not depend on host-specific paths

For details:
- `docs/determinism/canonicalization.md`
- `docs/determinism/hashing.md`
- `docs/determinism/determinism-contract.md`

---

## How to validate a bundle

A standard bundle contains:
- `schema.json` (instance)
- `manifest.json` (instance)
- `proof.json` (instance)

Validation steps:
1. Validate `schema.json` against `schemas/v1/schema.json`
2. Validate `manifest.json` against `schemas/v1/manifest.json`
3. Validate `proof.json` against `schemas/v1/proof.json`
4. Recompute `schemaHash` and compare with `manifest.hashes.schemaHash`
5. Recompute Merkle root and compare with `proof.root`

The CLI implements these checks:
```bash
signia verify --bundle ./out --strict
```

---

## Extending schema kinds

Schema kinds are not fixed to a small list. A plugin can define:
- new entity types
- new edge types
- additional deterministic attributes

Constraints:
- all attributes must be deterministic JSON
- do not embed timestamps or random values
- avoid environment-dependent identifiers

See:
- `docs/plugins/plugin-spec.md`
- `docs/plugins/writing-a-plugin.md`

---

## Examples

### Repository schema example
- `v1/examples/repo.schema.json`

### OpenAPI schema example
- `v1/examples/openapi.schema.json`

### Dataset schema example
- `v1/dataset.schema.json`

### Workflow schema example
- `v1/workflow.schema.json`

---

## Compatibility

Consumers should:
- treat unknown entity and edge types as opaque but preserved
- validate required fields
- avoid dropping unknown attributes during processing

---

## License

These schemas are part of the SIGNIA project and follow the repository license.
