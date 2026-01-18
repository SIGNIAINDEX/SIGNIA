
# Trust Boundaries

This document defines trust boundaries in SIGNIA: where untrusted data enters, where trust assumptions change, and how components isolate risk.

In SIGNIA, trust is minimized by design:
- inputs are untrusted
- outputs are verified
- on-chain anchoring is minimal
- determinism is enforced and treated as a security property

---

## 1) Why trust boundaries matter

SIGNIA processes artifacts from the real world (repos, specs, schemas, workflows). These inputs can be malformed, malicious, or simply unpredictable. Without clear trust boundaries:

- parsers become RCE/DoS vectors
- nondeterminism becomes an integrity vulnerability
- registry metadata becomes a social engineering surface
- hosted services become abuse targets

This document explicitly marks:
- what is untrusted
- what must be verified
- what must be isolated or constrained

---

## 2) System overview (boundary view)

```
                ┌─────────────────────────────────────────┐
                │            Untrusted World               │
                │ (files, repos, archives, URLs, payloads) │
                └─────────────────────────────────────────┘
                               │
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ Boundary A: Input Ingest / Normalization                      │
  │ - canonical paths, encodings, line endings                     │
  │ - size limits and sandbox policy                               │
  └─────────────────────────────────────────────────────────────┘
                               │
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ Boundary B: Parsing / Plugin Execution                         │
  │ - parse untrusted bytes                                         │
  │ - produce IR (still untrusted until canonicalized + validated)   │
  └─────────────────────────────────────────────────────────────┘
                               │
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ Boundary C: IR Validation + Canonicalization                   │
  │ - enforce schema invariants                                     │
  │ - normalize ordering + encoding                                 │
  │ - compute domain-separated hashes                               │
  └─────────────────────────────────────────────────────────────┘
                               │
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ Boundary D: Bundle Emission + Local Store                      │
  │ - content-addressed storage                                     │
  │ - bundles become verifiable artifacts                           │
  └─────────────────────────────────────────────────────────────┘
                               │
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ Boundary E: Verification                                       │
  │ - recompute canonical hashes                                    │
  │ - verify proof roots and manifest linkage                       │
  └─────────────────────────────────────────────────────────────┘
                               │
                               ▼
       ┌───────────────────────────────┐     ┌───────────────────┐
       │ Optional: On-chain Registry     │<───│ Boundary F: Publish │
       │ (minimal metadata + hash anchor)│     │ (signer required)  │
       └───────────────────────────────┘     └───────────────────┘
```

---

## 3) Trust boundary definitions

### Boundary A: Input ingest and normalization
**Untrusted:** everything outside SIGNIA (filesystem, archives, URLs, API payloads)

Responsibilities:
- establish a strict input root (workspace sandbox)
- normalize:
  - path separators
  - unicode normalization policy (if applicable)
  - newline normalization
  - encoding handling (UTF-8 policy)
- enforce limits:
  - max total bytes
  - max file size
  - max directory depth
  - max number of files
- define symlink policy:
  - deny symlinks (simplest)
  - or resolve only within root and validate canonical path
- define remote fetch policy:
  - default deny
  - allow only when pinned by immutable refs and/or content addressing

Trust rule:
- Inputs remain untrusted after normalization.
- Normalization is a deterministic security gate, not a trust elevation.

### Boundary B: Parsing and plugin execution
**Highest-risk boundary.**

Untrusted:
- parse targets (bytes)
- plugin configuration values (unless validated)
- plugin outputs (IR) until validated

Responsibilities:
- treat parsers as hostile input handlers
- isolate plugin execution:
  - timeouts
  - memory limits where feasible
  - maximum graph size for emitted IR
- disallow side effects by default:
  - no shell execution
  - no arbitrary filesystem access outside workspace
  - no network access unless explicitly enabled
- validate plugin output types strictly:
  - required fields
  - stable identity constraints
  - no duplicate node IDs
  - bounded attribute sizes

Trust rule:
- IR produced by plugins is not trusted until it passes IR validation.

### Boundary C: IR validation and canonicalization
This is the boundary where outputs become *structurally trustworthy*.

Untrusted:
- IR from plugins

Responsibilities:
- validate IR invariants:
  - node identity uniqueness
  - edge validity (references to existing nodes)
  - type constraints (well-formed)
  - consistent version markers
- apply canonicalization rules:
  - stable ordering for all collections
  - identifier normalization rules
  - canonical JSON encoding
  - domain-separated hashing
- compute:
  - schema hash
  - proof root
  - manifest hash (if used)

Trust rule:
- After canonicalization, schema bytes are stable and hashable.
- This is not a correctness proof of real-world truth; it is an integrity boundary.

### Boundary D: Bundle emission and local store
**Trust changes from “transient bytes” to “verifiable artifact.”**

Untrusted:
- filesystem can be tampered with after emission

Responsibilities:
- write bundles atomically where feasible
- store by content address (hash-based directories)
- prevent path traversal on import/export
- avoid mixing hashed and non-hashed data silently
- ensure the store never treats filenames as authoritative identifiers

Trust rule:
- A bundle is only trustworthy after verification, even if produced locally.
- The store provides convenience, not trust.

### Boundary E: Verification
Verification is the trust gate for consumers.

Untrusted:
- any bundle received from outside
- local bundles (could be corrupted, partial, or stale)
- registry metadata and off-chain indexes

Responsibilities:
- recompute hashes from canonical bytes
- validate manifest linkage:
  - manifest references schema hash and proof root correctly
- validate proof:
  - root is derived from the defined leaf set and ordering
- fail closed:
  - any mismatch is a hard failure
- provide actionable errors:
  - which file mismatched
  - expected vs actual hash

Trust rule:
- Verified bundles become trusted *as verifiable structure artifacts*.

### Boundary F: Publish to registry (optional)
Publishing is not verification; it is anchoring.

Untrusted:
- publisher-provided metadata
- publisher intent

Responsibilities:
- require signer for writes
- enforce idempotency (registering the same schema hash should be safe)
- constrain metadata size (avoid large on-chain writes)
- ensure PDA derivation is correct and collision-resistant
- prevent unauthorized overwrites

Trust rule:
- On-chain record proves a publisher anchored a hash at a time, not that the schema is correct.
- Consumers should still verify bundles and apply publisher policies.

---

## 4) Data classification and handling rules

### Hashed domains (security-critical)
These must be deterministic and treated as security boundaries:
- canonical schema bytes
- hash domains and canonical JSON encoding
- proof leaf sets and ordering rules
- manifest fields that affect verification

Rules:
- never include timestamps, randomness, or environment-specific paths
- never include secrets
- define explicit ordering for every collection

### Non-hashed metadata (informational)
Examples:
- display titles
- descriptions
- UI hints

Rules:
- must be explicitly marked as non-hashed
- must not influence verification outcomes
- treat as untrusted and subject to poisoning

---

## 5) Boundary-specific threat examples

### Example: Path traversal through archive extraction
If an archive contains `../../.ssh/id_rsa`, extraction must never write outside the workspace root.

Controls:
- canonical path checks
- deny absolute paths
- strip drive prefixes (Windows)
- reject symlinks by default

### Example: Nondeterministic ordering from filesystem traversal
If file enumeration depends on OS iteration order, schema hashes will drift.

Controls:
- stable sorting by normalized path
- remove platform-dependent separators
- normalize unicode and line endings

### Example: Registry metadata social engineering
A publisher sets a misleading label to impersonate an official project.

Controls:
- keep on-chain metadata minimal
- treat metadata as untrusted
- provide allowlists/endorsement policies off-chain

---

## 6) Implementation guidance (practical)

### Recommended isolation defaults
- deny network access for compilation by default
- deny symlinks by default
- parse only whitelisted file types per plugin
- set strict maximums for:
  - bytes
  - files
  - IR nodes/edges
- apply timeouts per stage:
  - ingest
  - plugin parse
  - canonicalize
  - proof build

### Recommended validation layers
- validate IR with an internal schema
- validate bundles with JSON Schema specs in `schemas/`
- verify hashes and proofs at the end of every compile
- enforce golden fixtures in CI for determinism

### Recommended registry constraints
- PDA seeds include schema hash and a domain tag
- cap metadata string lengths
- enforce record immutability after creation, except explicit status flags
- restrict upgrade authority and document governance

---

## 7) Boundary mapping to repository components

Typical mapping (recommended):
- `crates/signia-ingest`: Boundary A
- `crates/signia-plugins`: Boundary B
- `crates/signia-core`: Boundary C
- `crates/signia-store`: Boundary D
- `crates/signia-verify`: Boundary E
- `crates/signia-solana-client` + `programs/signia-registry`: Boundary F

---

## 8) Review checklist for boundary changes

Any PR that changes trust boundaries should answer:

- Which boundary changed and why?
- Did limits become looser? If yes, what compensating controls exist?
- Did hashing or canonicalization change? If yes:
  - is the change versioned?
  - are golden fixtures updated?
  - are migrations documented?
- Did plugin permissions change (filesystem/network)? If yes:
  - is the default still deny?
  - is the change scoped and auditable?
- Did registry constraints change? If yes:
  - is the account model still minimal?
  - are instruction constraints safe?

---

## 9) Summary

SIGNIA maintains a strict separation between:
- **untrusted inputs**
- **validated, canonicalized structures**
- **verifiable bundles**
- **optional on-chain anchors**

Verification is the primary trust gate. On-chain publication is an anchor, not proof.

Determinism is enforced because nondeterminism is a security failure in a hash-addressed system.
