
# Schema v1

This document specifies the **SIGNIA Schema v1** format: the canonical structure representation emitted as `schema.json` inside a SIGNIA bundle.

The schema is designed to be:
- deterministic (canonicalizable into stable bytes)
- verifiable (hash-addressed by schema hash)
- composable (referenceable by hash and linkable across structures)
- plugin-agnostic (shared IR across input types)

This document defines:
- fields and constraints
- identity model (node/edge/type IDs)
- canonical ordering requirements
- hashing domain linkage requirements

---

## 1) Overview

A SIGNIA bundle contains:
- `schema.json` (this document)
- `manifest.json` (compilation context and integrity links)
- `proof.json` (Merkle root and verification material)

`schema.json` describes a structural graph:
- **entities** (nodes)
- **relationships** (edges)
- **types** (structural type definitions)
- **constraints** (invariants)

The schema does not encode execution semantics. It encodes structure.

---

## 2) File identity and hashing

### 2.1 Schema hash (identifier)
The schema hash is computed over the canonical bytes of `schema.json` with the schema hash domain:

- `schema_hash = H( domain("signia:schema:v1") || canonical_bytes(schema.json) )`

Where:
- `H` is the repository-selected hash function (see `docs/determinism/hashing.md`)
- canonical bytes are defined by `docs/determinism/canonicalization.md`

### 2.2 Required header fields
`schema.json` MUST contain:
- `schema_version`: string, MUST be `"v1"`
- `hash_domain`: string, MUST be `"signia:schema:v1"`
- `schema_id`: string, SHOULD be the canonical display form of `schema_hash` (hex) once computed

---

## 3) JSON shape (top-level)

Top-level object:

```json
{
  "schema_version": "v1",
  "hash_domain": "signia:schema:v1",
  "schema_id": "<hex hash string>",
  "root": {
    "artifact": { ... },
    "graph": { ... },
    "types": { ... },
    "constraints": { ... }
  },
  "meta": { ... }
}
```

Notes:
- `meta` is optional and MUST NOT affect hashing unless explicitly included in hashed domains. Recommended: treat `meta` as non-hashed and store it outside hashed view (or keep it minimal and deterministic).

---

## 4) Artifact descriptor

`root.artifact` describes what structure is modeling (not the compilation context; that belongs to the manifest).

```json
"artifact": {
  "kind": "repo|openapi|dataset|workflow|config|spec|unknown",
  "name": "<string>",
  "namespace": "<string>",
  "ref": "<string>",
  "labels": ["<string>", "..."]
}
```

Field rules:
- `kind` MUST be one of the allowed enum values (extendable in future versions).
- `name` SHOULD be a stable logical name.
- `namespace` MAY represent an org or domain.
- `ref` is a logical reference for humans (not necessarily the pinned immutable ref).
- `labels` MUST be sorted lexicographically in canonical form.

Canonicalization:
- All strings must be normalized per string/path rules if they include paths or identifiers.
- `labels` must be sorted and deduplicated.

---

## 5) Graph model

The graph is a structural directed graph.

```json
"graph": {
  "entities": [ { ... }, ... ],
  "edges": [ { ... }, ... ],
  "indexes": { ... }
}
```

Rules:
- `entities` MUST contain unique `id` values.
- `edges` MUST reference existing entity IDs in `from` and `to`.
- `indexes` is optional and should be treated as non-hashed unless specified (indexing can be derived).

---

## 6) Entity model (nodes)

Entity object:

```json
{
  "id": "ent:<kind>:<stable-id>",
  "kind": "<string>",
  "name": "<string>",
  "path": "<normalized path or null>",
  "digest": "<hex hash or null>",
  "attrs": { ... },
  "tags": ["<string>", "..."]
}
```

Field definitions:
- `id` (required): stable identifier for this entity.
- `kind` (required): entity type category (e.g., `module`, `endpoint`, `table`, `section`).
- `name` (required): human-readable name, deterministic where possible.
- `path` (optional): normalized logical path.
- `digest` (optional): hash of underlying content bytes if a plugin defines it (content hashing is optional).
- `attrs` (optional): entity attributes; must be canonicalizable.
- `tags` (optional): sorted list of tags.

Identity rules:
- `id` MUST be stable for the same normalized input.
- Plugins MUST document entity ID strategy.

Canonical ordering:
- Entities MUST be sorted by `(kind, id)`.

Constraints:
- `attrs` MUST be a JSON object with sorted keys and canonicalizable values.
- Avoid floats in hashed domains.

---

## 7) Edge model (relationships)

Edge object:

```json
{
  "id": "edge:<relation>:<from-id>:<to-id>:<tiebreaker>",
  "relation": "<string>",
  "from": "<entity id>",
  "to": "<entity id>",
  "attrs": { ... }
}
```

Field definitions:
- `relation` (required): relationship type (e.g., `depends_on`, `imports`, `references`).
- `from`/`to` (required): entity IDs.
- `id` (required): stable edge identity; MUST be deterministic.

Canonical ordering:
- Edges MUST be sorted by `(relation, from, to, id)`.

Constraints:
- `from` and `to` MUST exist in `entities`.
- `attrs` must be canonicalizable.

---

## 8) Types

Types define structural shapes used by entities or constraints.

Top-level types container:

```json
"types": {
  "definitions": [ { ... }, ... ]
}
```

Type definition object:

```json
{
  "id": "type:<kind>:<stable-id>",
  "kind": "object|array|string|number|integer|boolean|null|enum|ref|union",
  "name": "<string>",
  "definition": { ... },
  "attrs": { ... }
}
```

Rules:
- Types MUST be sorted by `(kind, id)`.
- `kind` is an enum for v1, extensible in later versions.

### 8.1 Object type
Example:

```json
{
  "id": "type:object:User",
  "kind": "object",
  "name": "User",
  "definition": {
    "properties": [
      { "name": "id", "type": "type:string:uuid", "required": true },
      { "name": "email", "type": "type:string:email", "required": false }
    ],
    "additional_properties": false
  }
}
```

Canonical ordering:
- `properties` MUST be sorted by property name.
- Object keys in `definition` must be key-sorted.

### 8.2 Array type
Example:

```json
{
  "id": "type:array:Users",
  "kind": "array",
  "definition": {
    "items": "type:object:User",
    "min_items": 0,
    "max_items": null
  }
}
```

### 8.3 Enum type
Example:

```json
{
  "id": "type:enum:Status",
  "kind": "enum",
  "definition": {
    "values": ["active", "disabled", "pending"]
  }
}
```

Canonical ordering:
- `values` MUST be sorted lexicographically unless the source semantics require order. Prefer stable sorted enums.

### 8.4 Ref type
Example:

```json
{
  "id": "type:ref:User",
  "kind": "ref",
  "definition": {
    "ref": "type:object:User"
  }
}
```

---

## 9) Constraints

Constraints encode invariants over the graph and types.

Top-level constraints container:

```json
"constraints": {
  "rules": [ { ... }, ... ]
}
```

Constraint rule object:

```json
{
  "id": "c:<kind>:<stable-id>",
  "kind": "<string>",
  "scope": {
    "entities": ["<entity id>", "..."],
    "types": ["<type id>", "..."]
  },
  "predicate": { ... },
  "severity": "info|warn|error",
  "attrs": { ... }
}
```

Canonical ordering:
- Constraints MUST be sorted by `(kind, id)`.

Rules:
- `scope.entities` and `scope.types` MUST be sorted and deduplicated.
- `severity` is a small enum.
- `predicate` MUST be canonicalizable.

Examples:
- compatibility invariants
- required entity existence
- schema version constraints
- disallowed edges

---

## 10) Meta (optional)

`meta` may include non-critical information:
- display name
- description
- source hints

Rules:
- If included in hashed schema bytes, meta must be deterministic and canonicalizable.
- Recommended: keep `meta` minimal or move non-hashed metadata to a separate file or manifest metadata domain.

---

## 11) Canonicalization requirements (normative)

Schema v1 is valid only if canonicalization requirements are met.

Normative rules:
- JSON key ordering: lexicographic by Unicode code point
- No insignificant whitespace
- UTF-8 encoding
- Collections sorted as specified:
  - entities by `(kind, id)`
  - edges by `(relation, from, to, id)`
  - types by `(kind, id)`
  - constraints by `(kind, id)`
- All internal lists that represent sets MUST be sorted and deduplicated:
  - artifact labels
  - entity tags
  - constraint scopes

---

## 12) Validation rules (normative)

A schema MUST fail validation if:
- `schema_version` is not `"v1"`
- `hash_domain` is incorrect
- entity IDs are not unique
- an edge references a missing entity
- a type ID is duplicated
- ordering constraints are violated (when validating canonical form)
- required fields are missing

---

## 13) Extensibility

Schema v1 is designed to evolve.

Rules:
- Additive changes are preferred (new optional fields).
- Breaking changes require Schema v2 with new hash domains.
- New kinds for artifacts/entities/types/constraints should be introduced carefully and documented.

---

## 14) Example (minimal)

Minimal schema example (illustrative):

```json
{
  "schema_version": "v1",
  "hash_domain": "signia:schema:v1",
  "schema_id": "0123...abcd",
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

---

## 15) Related documents

- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
- Determinism contract: `docs/determinism/determinism-contract.md`
- Manifest spec: `docs/schemas/manifest-v1.md`
- Proof spec: `docs/schemas/proof-v1.md`
