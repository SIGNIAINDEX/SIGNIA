# Determinism Contract

This document defines the determinism contract for SIGNIA: the explicit rules and guarantees that ensure identical inputs produce identical outputs (byte-for-byte), enabling reliable hashing, proofs, and independent verification.

Determinism is not a convenience feature in SIGNIA. It is a security property.

---

## 1) Contract statement

Given:
- the same input artifact bytes (or pinned immutable reference)
- the same SIGNIA version
- the same plugin set and plugin versions
- the same normalization policy and configuration
- the same canonicalization and hashing specifications

SIGNIA must produce:
- identical canonical `schema.json` bytes
- identical canonical `manifest.json` bytes (for hashed fields)
- identical `proof.json` (root and proof material derived from defined leaves)
- identical schema hash, manifest hash (if used), and proof root

**Same input → same output (byte-for-byte).**

---

## 2) Determinism scope

Determinism applies to:

- Canonical bytes used for hashing:
  - schema canonical bytes
  - manifest canonical bytes (as specified)
  - proof leaf encodings
- Hashing:
  - domain-separated hash definitions
  - leaf hashing and Merkle root derivation
- Ordering:
  - every collection (maps, sets, lists) in hashed domains
- Normalization:
  - paths, line endings, encoding rules, timestamps
- Plugin outputs:
  - IR must be deterministic for the same normalized input

Determinism does not require:
- identical performance metrics
- identical logs
- identical non-hashed metadata (unless explicitly specified)

---

## 3) Inputs: what must be pinned

### 3.1 Immutable references are required for reproducibility
Acceptable pinning strategies include:
- commit SHA (for VCS sources)
- content checksum (for archives and files)
- explicit versioned releases with checksums

Floating references are allowed only if they are converted into pinned inputs at compile time:
- branch names (e.g., `main`)
- mutable URLs
- “latest” tags

If a floating ref is used, the manifest must record the resolved immutable reference.

### 3.2 Network access policy
Default:
- no network access during compilation

If network access is enabled:
- every fetched input must be content-addressed or pinned
- caches must not change results
- the manifest must record the resolved immutable identifiers

---

## 4) Normalization contract (input canonicalization)

Normalization removes environment variance before parsing.

### 4.1 Paths
- All paths must be represented in normalized POSIX form using `/` separators in hashed domains.
- Absolute paths must never appear in hashed domains.
- Input roots must be mapped to a logical root (e.g., `/` or `repo://`).

### 4.2 Newlines and encoding
- Text inputs must be normalized to LF (`\n`) for hashing domains.
- UTF-8 is the canonical encoding.
- If an input is not valid UTF-8 and the plugin expects text, the plugin must:
  - reject with a deterministic error, or
  - define a deterministic byte-to-text mapping strategy (must be documented).

### 4.3 Timestamps and environment-derived values
- Wall-clock timestamps must never influence hashed domains.
- If timestamps exist in inputs (e.g., metadata files), plugins must:
  - ignore them, or
  - normalize them into a deterministic placeholder, or
  - treat them as non-hashed metadata.

### 4.4 Symlinks
Default recommended policy:
- deny symlinks

If symlinks are allowed:
- resolve only within the input root
- validate canonical path containment
- define deterministic resolution behavior and record policy version in the manifest

---

## 5) IR determinism contract

Plugins produce IR. IR is untrusted until validated and canonicalized.

### 5.1 IR must be deterministic
For the same normalized input and plugin config, plugins must produce identical IR.

Plugins must not:
- iterate using filesystem order without sorting
- depend on locale/timezone
- generate random IDs
- use nondeterministic concurrency for ordering
- include host-specific paths or usernames

### 5.2 Stable identities
Every entity and edge must have a stable identity strategy documented by the plugin.

Examples:
- entity ID derived from normalized path + kind
- edge ID derived from (from_id, to_id, relation_type)

### 5.3 Bounded outputs
Plugins must enforce bounds:
- maximum nodes/edges
- maximum attribute sizes
- maximum recursion depth

Bounds must produce deterministic failures.

---

## 6) Canonical JSON encoding (byte-level contract)

All hashed JSON documents must be serialized in a canonical way.

### 6.1 Key ordering
- Object keys must be sorted lexicographically by Unicode code point.
- No “insertion order” reliance.

### 6.2 Whitespace
- No insignificant whitespace.
- No trailing spaces.
- Use `:` and `,` without extra spaces.

### 6.3 Numbers
- Integers encoded in base-10 without leading zeros (except `0`).
- Floats, if allowed, must follow a strict canonical format.
  - Recommended: avoid floats in hashed domains; represent as rational or string if needed.

### 6.4 Strings
- Use JSON standard escaping.
- No ambiguous unicode normalization at encoding time unless explicitly defined.

### 6.5 Null/boolean
- Standard JSON literals: `null`, `true`, `false`.

### 6.6 UTF-8 output
- Canonical bytes must be UTF-8 encoded.

---

## 7) Hashing contract

### 7.1 Hash function
The hash function must be documented and stable for a given major version.

Recommended:
- SHA-256 or BLAKE3 (choose one per spec; do not mix without domain separation)

The hash function is part of the determinism contract. Changing it requires a version bump.

### 7.2 Domain separation
Every hash must include a domain tag prefix.

Examples (illustrative):
- `signia:schema:v1`
- `signia:manifest:v1`
- `signia:proof:v1`
- `signia:leaf:entity:v1`
- `signia:leaf:edge:v1`

### 7.3 Hash inputs
Hashes must be computed over canonical bytes.

Rules:
- never hash in-memory structures without canonical serialization
- never hash debug outputs
- never hash non-deterministic representations

---

## 8) Proof construction contract

### 8.1 Leaf set definition
Proof leaves must be defined by the spec:
- what constitutes a leaf
- how leaves are encoded (canonical bytes)
- how leaves are ordered

### 8.2 Leaf ordering
Leaf ordering must be deterministic and stable:
- sort by (leaf_type, stable_id) or an equivalent stable key
- define a total ordering (no ties)

### 8.3 Merkle tree construction
Tree construction must be deterministic:
- define whether odd leaves are duplicated, promoted, or padded
- define node hashing domain and concatenation rules
- define root representation

### 8.4 Proof material
If inclusion proofs are included:
- define sibling ordering
- define direction markers (left/right) deterministically
- encode proofs canonically (JSON canonical encoding or binary spec)

---

## 9) Error determinism contract

Failures must be deterministic:
- same input → same error category and message class

Guidelines:
- errors should include stable identifiers, not host-dependent paths
- avoid embedding OS-specific errno strings in stable outputs
- provide structured error codes for programmatic handling

Non-goal:
- exact byte-for-byte matching of logs across environments

---

## 10) Determinism testing requirements

### 10.1 Golden fixtures
For each plugin and core pipeline:
- commit at least one realistic fixture
- commit expected canonical outputs
- CI must validate byte-for-byte equality

### 10.2 Cross-run checks
Run compilation twice in CI and compare:
- schema bytes
- schema hash
- proof root

### 10.3 Cross-platform checks
At minimum:
- Linux and macOS builds validate determinism fixtures
- Windows is recommended if path handling is supported

### 10.4 Negative tests
Mutate bundle files and ensure verification fails:
- schema tampering
- manifest tampering
- proof tampering

---

## 11) Change control and versioning

Changes that affect determinism require:
- an explicit version bump where relevant:
  - schema version
  - manifest version
  - proof version
  - hash domain version
- updated specs and JSON schemas
- updated fixtures
- documented migration notes

Security-sensitive changes include:
- canonical JSON encoding rules
- ordering rules
- hashing domains
- proof leaf definitions
- path normalization policies

---

## 12) Consumer obligations

Consumers must:
- verify bundles before trusting them
- apply policy for publisher allowlists if needed
- avoid relying on non-hashed metadata for security decisions
- pin inputs in CI and record immutable refs

---

## 13) Summary

SIGNIA’s determinism contract ensures:
- stable canonical bytes
- stable hashes and proof roots
- independent verification without trusting operators

Determinism failures are treated as integrity vulnerabilities and must be fixed with priority.
