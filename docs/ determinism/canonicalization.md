# Canonicalization

This document specifies canonicalization rules used by SIGNIA. Canonicalization transforms an intermediate structural model into a **canonical, deterministic byte representation** suitable for hashing, proofs, and independent verification.

Canonicalization is part of the security model: ambiguous encoding or ordering can enable substitution attacks and nondeterministic hash drift.

---

## 1) Canonicalization goals

1. **Deterministic bytes**: identical logical structures yield identical bytes.
2. **Stable hashing**: schema hashes and proof roots are computed over canonical bytes.
3. **Cross-platform consistency**: output is independent of OS, filesystem order, locale, and time.
4. **Verifiability**: third parties can recompute canonical bytes and confirm integrity.

---

## 2) Canonicalization scope

Canonicalization applies to:
- schema documents (`schema.json`)
- manifest documents (`manifest.json`) for hashed fields
- proof leaf encodings and proof documents (`proof.json`) where specified
- any internal encodings used as hash input

Canonicalization does not apply to:
- logs
- human-readable summaries
- non-hashed metadata (unless explicitly stated)

---

## 3) Canonical JSON encoding

SIGNIA uses a canonical JSON encoding for hashed domains.

### 3.1 JSON object key ordering
- Keys MUST be sorted lexicographically by Unicode code point.
- Nested objects follow the same rule.
- No reliance on insertion order.

### 3.2 Whitespace and formatting
- No insignificant whitespace is permitted.
- Use `:` and `,` with no extra spaces.
- No trailing newline requirements unless explicitly specified; if present, it must be consistent (recommended: no trailing newline).

### 3.3 String encoding
- Output bytes MUST be UTF-8.
- Escape sequences MUST follow JSON standard.
- Avoid emitting unnecessary escapes where not required.
- Unicode normalization: do not transform string content unless the field’s normalization rules require it (see Section 4).

### 3.4 Numbers
- Integers MUST be base-10 without leading zeros (except `0`).
- Floats SHOULD be avoided in hashed domains.
  - If floats are unavoidable, represent them as canonical strings or rationals with explicit rules.

### 3.5 Booleans and null
- Standard JSON literals: `true`, `false`, `null`.

---

## 4) Identifier and string normalization

Many fields must be normalized before canonical JSON encoding to avoid platform variance.

### 4.1 Path normalization
For any field defined as a path in hashed domains:

- Convert separators to `/`.
- Remove drive letters (Windows) and map input roots to logical roots.
- Remove `.` segments.
- Resolve `..` segments within root; reject escapes outside root.
- Prohibit absolute host paths.
- Normalize repeated separators (`//` → `/`) where applicable.
- Define a stable root prefix:
  - examples: `repo:/`, `artifact:/`, `input:/`
  - choose one and apply consistently.

### 4.2 Newline normalization (text fields)
If a field is derived from text content and the content affects structure:
- Normalize newlines to LF (`\n`) before hashing.
- Do not include raw CRLF in hashed domains.

### 4.3 Case normalization
Case rules MUST be explicit per field. Default:
- do not change case unless the semantics require it.

Examples:
- IDs might be case-sensitive; do not lowercase.
- certain formats may define case-insensitive identifiers; normalize to lowercase with documented rules.

### 4.4 Unicode normalization
Unicode normalization is a policy decision.
- Default recommendation: do not apply Unicode normalization globally.
- If applied, it MUST be applied consistently and documented (e.g., NFC on specific identifier fields).

The normalization policy version MUST be recorded in the manifest.

### 4.5 Whitespace trimming
Trimming rules MUST be explicit per field. Default:
- do not trim unless the field is defined as a label with trimming semantics.

---

## 5) Collection ordering rules

Canonicalization requires a total ordering for every collection.

### 5.1 Objects vs arrays
- JSON objects are for unordered maps; canonical order is key-sorted.
- JSON arrays preserve order; arrays must be explicitly sorted where order is not semantically meaningful.

### 5.2 Sorting rules
When sorting arrays in hashed domains, define:
- sort key fields
- tie-breakers
- stable behavior for duplicates

General recommendation:
- sort by `(type, stable_id, secondary_key...)` with a complete tie-breaker chain.

### 5.3 No ambiguous ordering
Avoid fields like “list of objects” without sort rules. If order is not meaningful:
- convert to an object keyed by stable ID, or
- define array sorting rules.

---

## 6) Canonical schema rules (schema.json)

This section describes canonicalization requirements for schema content.

### 6.1 Stable schema header
The schema MUST include:
- `schema_version` (e.g., `"v1"`)
- `hash_domain` (explicit)
- `root_id` or equivalent primary identity anchor (if applicable)

### 6.2 Entity normalization
Entities MUST have:
- stable `id`
- `kind` or `type`
- deterministic attributes

Canonical entity ordering:
- entities MUST be sorted by `(kind, id)` unless a stronger rule is specified.

### 6.3 Edge normalization
Edges MUST have:
- stable `from`
- stable `to`
- relation type
- optional attributes, normalized

Canonical edge ordering:
- edges MUST be sorted by `(relation, from, to, edge_id)` where `edge_id` is a deterministic tie-breaker.

### 6.4 Type definitions
Types MUST be:
- fully explicit (no implicit defaults in hashed domains unless specified)
- referenced by stable IDs

Canonical type ordering:
- type definitions sorted by `(type_kind, type_id)`.

### 6.5 Constraints and invariants
Constraints MUST be:
- explicit in canonical form
- ordered deterministically (sorted lists or objects)

---

## 7) Canonical manifest rules (manifest.json)

The manifest captures compilation context and integrity links.

### 7.1 Required manifest fields (hashed)
Recommended hashed manifest fields:
- `manifest_version`
- `schema_hash`
- `proof_root`
- `input_descriptor` (pinned ref and normalization policy)
- `toolchain` versions (compiler and plugins)
- `dependencies` (hash-addressed references)

Fields that are not hashed MUST be clearly separated and marked.

### 7.2 Dependency ordering
Dependencies MUST be ordered deterministically, typically by:
- `(dependency_type, dependency_id)` or `(schema_hash)`.

### 7.3 Input pinning representation
The manifest MUST represent:
- input type (repo/archive/file/url)
- immutable reference (commit SHA/checksum)
- normalization policy version
- plugin config hash (if applicable)

All strings used here must follow normalization rules.

---

## 8) Canonical proof rules (proof.json)

Proofs bind leaf sets to a root hash.

### 8.1 Leaf encoding
Each leaf MUST have:
- leaf type (entity/edge/type/constraint)
- stable identity key
- canonical bytes or canonical JSON representation
- leaf hash in a leaf hash domain

### 8.2 Leaf ordering
Leaves MUST be sorted by a total ordering:
- `(leaf_type, stable_id)` with a tie-breaker if needed.

### 8.3 Tree construction rules
The proof specification MUST define:
- hash domain for internal nodes
- how pairs are combined (left||right concatenation)
- odd leaf handling (duplicate or promote) — choose one and standardize
- root encoding (hex/base58/etc.) — define explicitly

---

## 9) Hashing and domains

Canonicalization is inseparable from hashing.

### 9.1 Domain tags
Every hash MUST include a domain tag prefix.

Examples (illustrative):
- `signia:schema:v1`
- `signia:manifest:v1`
- `signia:proof:v1`
- `signia:node:v1`
- `signia:edge:v1`

### 9.2 Hash input bytes
Hash inputs MUST be canonical bytes, never in-memory structures.

### 9.3 Encoding for display
When hashes are displayed:
- choose a single representation (hex or base58)
- define it in the spec
- avoid mixing formats without labeling

---

## 10) Implementation guidance (practical)

### 10.1 Canonical JSON library behavior
If using a JSON library:
- ensure stable key ordering (do not rely on default map order)
- ensure deterministic float encoding (or ban floats)
- ensure UTF-8 output and stable escaping

### 10.2 Avoid hidden nondeterminism
Common pitfalls:
- iteration over hash maps without sorting
- filesystem traversal order
- locale-sensitive comparisons
- time-based defaults in metadata
- concurrent processing without stable merge ordering

### 10.3 Deterministic merge strategy
When combining multiple sources:
- define stable merge rules
- define precedence
- record merge policy in the manifest

### 10.4 Versioning
Any change to canonicalization rules requires:
- version bump (schema/manifest/proof or rule version)
- updated JSON schemas
- updated golden fixtures
- migration notes

---

## 11) Compliance tests

A canonicalization implementation MUST be covered by:
- golden fixtures: expected canonical bytes committed in repo
- round-trip tests: parse → canonicalize → verify
- negative tests: tampering breaks verification
- cross-run tests: compile twice yields identical bytes/hashes

---

## 12) Summary

Canonicalization in SIGNIA ensures:
- stable bytes for hashing
- stable ordering for structure elements
- cross-platform reproducibility
- independent verification without trust in operators

Ambiguous encoding is treated as a security issue in a hash-addressed system.
