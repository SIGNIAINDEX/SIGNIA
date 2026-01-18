
# Examples

This document contains practical examples of SIGNIA v1 bundle artifacts:
- `schema.json` examples (Schema v1)
- `manifest.json` examples (Manifest v1)
- `proof.json` examples (Proof v1)
- end-to-end example bundles you can verify

All JSON blocks below are **illustrative** and may be shortened for readability. When used as fixtures, ensure they are canonicalized per:
- `docs/determinism/canonicalization.md`
- `docs/determinism/hashing.md`

---

## 1) Minimal OpenAPI-derived structure

This example models a single endpoint extracted from an OpenAPI document.

### 1.1 schema.json (minimal)

```json
{
  "schema_version": "v1",
  "hash_domain": "signia:schema:v1",
  "schema_id": "0123012301230123012301230123012301230123012301230123012301230123",
  "root": {
    "artifact": {
      "kind": "openapi",
      "name": "Example API",
      "namespace": "example",
      "ref": "v1",
      "labels": ["api", "openapi"]
    },
    "graph": {
      "entities": [
        {
          "id": "ent:endpoint:GET_/health",
          "kind": "endpoint",
          "name": "GET /health",
          "path": null,
          "digest": null,
          "attrs": { "method": "GET", "route": "/health" },
          "tags": ["public"]
        }
      ],
      "edges": [],
      "indexes": {}
    },
    "types": { "definitions": [] },
    "constraints": { "rules": [] }
  },
  "meta": {}
}
```

Notes:
- `schema_id` is shown as a placeholder 32-byte hash in hex.
- `labels` and `tags` are sorted.

### 1.2 manifest.json (minimal)

```json
{
  "manifest_version": "v1",
  "hash_domain": "signia:manifest:v1",
  "bundle": {
    "schema_hash": "0123012301230123012301230123012301230123012301230123012301230123",
    "proof_root": "4567456745674567456745674567456745674567456745674567456745674567",
    "manifest_hash": null,
    "schema_version": "v1",
    "proof_version": "v1",
    "created_by": {
      "compiler": "signia",
      "compiler_version": "0.1.0",
      "build": { "git_commit": null, "build_profile": "release", "target": null }
    }
  },
  "input": {
    "source": {
      "kind": "file",
      "uri": "artifact:/openapi.json",
      "resolved": { "commit": null, "checksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "etag": null }
    },
    "descriptor": {
      "descriptor_version": "v1",
      "descriptor_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "fields": {
        "plugin": "openapi",
        "plugin_version": "0.1.0",
        "normalization_policy": "v1",
        "canonicalization_rules": "v1"
      }
    }
  },
  "toolchain": {
    "compiler": {
      "name": "signia",
      "version": "0.1.0",
      "hash_function": "sha256",
      "canonicalization": { "rules_version": "v1" }
    },
    "plugins": [
      { "name": "openapi", "version": "0.1.0", "config_hash": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc", "notes": null }
    ]
  },
  "policies": {
    "normalization": {
      "policy_version": "v1",
      "path_root": "artifact:/",
      "newline": "lf",
      "encoding": "utf-8",
      "symlinks": "deny",
      "network": "deny"
    },
    "limits": {
      "max_total_bytes": 268435456,
      "max_file_bytes": 10485760,
      "max_files": 20000,
      "max_depth": 64,
      "max_nodes": 200000,
      "max_edges": 400000,
      "timeout_ms": 300000
    }
  },
  "dependencies": { "schemas": [] },
  "non_hashed": { "display": { "title": "Example API", "description": "" }, "annotations": { "publisher_label": null, "tags": [] } }
}
```

Notes:
- `checksum` is a placeholder.
- `descriptor_hash` and `config_hash` are placeholders.

### 1.3 proof.json (minimal)

```json
{
  "proof_version": "v1",
  "hash_domain": "signia:proof:v1",
  "hash_function": "sha256",
  "root": {
    "root_hash": "4567456745674567456745674567456745674567456745674567456745674567",
    "root_domain": "signia:proof-root:v1",
    "tree": { "node_domain": "signia:merkle:node:v1", "odd_leaf_rule": "duplicate_last", "arity": 2 }
  },
  "leaves": {
    "leaf_set": { "leaf_ordering": "type_then_id", "leaf_count": 1, "leaf_commitment": null },
    "items": [
      {
        "kind": "entity",
        "id": "ent:endpoint:GET_/health",
        "hash": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "projection": {
          "id": "ent:endpoint:GET_/health",
          "kind": "endpoint",
          "name": "GET /health",
          "path": null,
          "digest": null,
          "attrs": { "method": "GET", "route": "/health" },
          "tags": ["public"]
        }
      }
    ]
  },
  "inclusion_proofs": [],
  "meta": {}
}
```

---

## 2) Repository structure example (import graph)

This example models a small codebase structure: two modules and an import edge.

### 2.1 schema.json (repo graph)

```json
{
  "schema_version": "v1",
  "hash_domain": "signia:schema:v1",
  "schema_id": "1111111111111111111111111111111111111111111111111111111111111111",
  "root": {
    "artifact": {
      "kind": "repo",
      "name": "tiny-repo",
      "namespace": "example",
      "ref": "commit:deadbeef",
      "labels": ["repo"]
    },
    "graph": {
      "entities": [
        {
          "id": "ent:module:src/main.ts",
          "kind": "module",
          "name": "src/main.ts",
          "path": "artifact:/src/main.ts",
          "digest": "2222222222222222222222222222222222222222222222222222222222222222",
          "attrs": { "language": "typescript" },
          "tags": ["code"]
        },
        {
          "id": "ent:module:src/util.ts",
          "kind": "module",
          "name": "src/util.ts",
          "path": "artifact:/src/util.ts",
          "digest": "3333333333333333333333333333333333333333333333333333333333333333",
          "attrs": { "language": "typescript" },
          "tags": ["code"]
        }
      ],
      "edges": [
        {
          "id": "edge:imports:ent:module:src/main.ts:ent:module:src/util.ts:0",
          "relation": "imports",
          "from": "ent:module:src/main.ts",
          "to": "ent:module:src/util.ts",
          "attrs": { "spec": "./util" }
        }
      ],
      "indexes": {}
    },
    "types": { "definitions": [] },
    "constraints": { "rules": [] }
  },
  "meta": {}
}
```

---

## 3) Constraint example (compatibility rule)

This example includes a simple constraint: a schema requires at least one endpoint entity.

```json
{
  "id": "c:require-kind:endpoint",
  "kind": "require_kind",
  "scope": { "entities": [], "types": [] },
  "predicate": { "kind": "endpoint", "min_count": 1 },
  "severity": "error",
  "attrs": {}
}
```

---

## 4) Inclusion proof example (single leaf)

This example shows what an inclusion proof might look like for a leaf.

```json
{
  "kind": "entity",
  "id": "ent:endpoint:GET_/health",
  "leaf_hash": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  "path": [
    { "side": "right", "hash": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" },
    { "side": "left", "hash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" }
  ]
}
```

Notes:
- `path` is ordered from leaf to root.
- `side` indicates sibling position relative to the current hash at each step.

---

## 5) How to validate these examples locally

Using the SIGNIA CLI (once available in this repository):

- Compile:
  - `signia compile --input ./openapi.json --plugin openapi --out ./bundle`
- Verify:
  - `signia verify ./bundle`
- Inspect:
  - `signia inspect ./bundle --format json`
- Print hash:
  - `signia hash ./bundle/schema.json`

Verification steps recompute:
- schema canonical bytes and schema hash
- leaf hashes and Merkle root
- manifest linkage integrity

---

## 6) Fixture guidance (for repository maintainers)

When adding fixtures:
- store fixture input under `fixtures/<name>/input/`
- store expected outputs under `fixtures/<name>/expected/`
- ensure expected JSON files are canonical (no extra whitespace, keys sorted)
- record expected hashes under `fixtures/<name>/expected/hashes.json`

CI should:
- compile fixture twice
- compare outputs byte-for-byte
- run `verify` and fail closed on mismatches

---

## 7) Related documents

- Schema spec: `docs/schemas/schema-v1.md`
- Manifest spec: `docs/schemas/manifest-v1.md`
- Proof spec: `docs/schemas/proof-v1.md`
- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
