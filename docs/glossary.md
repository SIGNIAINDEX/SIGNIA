
# Glossary

This glossary defines key terms used throughout the SIGNIA documentation. Terms are written with SIGNIA's structure-first and determinism-first approach in mind.

---

## Artifact
An input object SIGNIA can ingest and compile (e.g., a repository, API schema, dataset schema, workflow graph, configuration model, or specification document). An artifact is not necessarily stored on-chain.

---

## Bundle
A deterministic output package emitted by SIGNIA, typically containing:
- `schema.json` (canonical schema)
- `manifest.json` (metadata and compilation context)
- `proof.json` (verification material)
A bundle may be stored locally, distributed off-chain, and optionally anchored on-chain via a registry record.

---

## Canonical Form
A normalized representation with stable ordering and encoding rules such that identical inputs produce identical bytes. Canonical form is required for reliable hashing and independent verification.

---

## Canonicalization
The process of transforming an intermediate structural model into canonical form by applying deterministic rules:
- ordering of collections
- identifier normalization
- platform-independent path normalization
- stable JSON encoding
- domain-separated hashing

---

## Compiler
The off-chain system that transforms an artifact into a canonical, verifiable structure bundle.

---

## Compilation
The deterministic pipeline that converts an artifact into a bundle:
Parse → Infer → Canonicalize → Compile → (Verify) → (Publish)

---

## Compilation Context
All non-content information required to reproduce a compilation, typically captured in the manifest:
- input descriptor (source, pinned reference)
- tool versions
- normalization policies
- plugin configuration

---

## Content
The literal bytes of an artifact (files, text, data). SIGNIA is structure-first: it does not attempt to mirror or host content as a primary goal. Content may be referenced, hashed, or included only insofar as it is needed to construct verifiable structure.

---

## Content-addressed
A storage and identification approach where objects are addressed by a hash of their canonical bytes (or a defined hash domain), rather than by a mutable name or location.

---

## Determinism
The property that for the same inputs and policies, SIGNIA produces the same outputs:
same input → same output (byte-for-byte), independent of environment.

---

## Determinism Rules
The explicit, versioned rules that define canonical ordering, normalization, and hashing behavior. Determinism rules must be documented and enforced by tests.

---

## Domain Separation
A hashing technique that prevents cross-context collisions by including a unique prefix or domain label in hash computation (e.g., `schema:v1`, `manifest:v1`, `proof:v1`).

---

## Edge
A directed relationship between two nodes in a structural graph (e.g., dependency, reference, import, includes, version link). Edges must have stable identity and semantics.

---

## Entity
A structural object within an IR or schema (e.g., a package, module, endpoint, table, document section). Entities are typically represented as nodes.

---

## Golden Fixture
A test artifact used to ensure determinism by comparing produced outputs (hashes and canonical bytes) against known, committed reference outputs.

---

## Hash
A cryptographic digest computed over canonical bytes within a specific domain. Hashes are used to identify schemas, manifests, proof roots, and registries.

---

## Hash Domain
A specific context for hashing with a unique domain prefix. SIGNIA uses domain separation to ensure the same bytes in different contexts cannot be substituted.

---

## Hash Root
A single digest representing a set of leaves (e.g., via a Merkle tree). Used for verification and on-chain anchoring.

---

## Identifier
A stable name or key used within a schema/IR. Identifiers must be normalized to avoid environment-specific variance (e.g., path separators, case rules, escaping).

---

## Inclusion Proof
A proof that a particular leaf belongs to a Merkle tree with a known root. Used to verify partial integrity for large bundles.

---

## Input Descriptor
A deterministic descriptor of the artifact source and pinning:
- source type (file/repo/url)
- immutable ref (commit SHA, checksum)
- normalization policy version
- plugin configuration hash
This is recorded in the manifest to enable reproducibility.

---

## Intermediate Representation (IR)
The common structural model produced by plugins and consumed by the compiler. The IR is versioned and designed for deterministic canonicalization.

---

## Leaf
A canonical, hashed element included in proof construction (e.g., an entity record, an edge record, or a schema fragment). Leaves have a stable ordering and encoding.

---

## Manifest
A canonical document capturing compilation context and integrity relationships:
- inputs and pinning
- tool versions and policies
- dependency links
- hashes of schema/proof
The manifest is part of the verifiable bundle.

---

## Merkle Tree
A tree constructed from hashed leaves that produces a root hash. Used to provide integrity proofs and partial verification.

---

## Metadata
Non-structural descriptive fields such as titles or descriptions. Metadata may be hashed or non-hashed depending on whether it is part of the canonical schema domain.

---

## Node
A unit in a structural graph, representing an entity. Nodes have stable identity and typed attributes.

---

## Normalization
The process of removing environment variance from inputs (paths, encodings, line endings, timestamps). Normalization precedes parsing and is part of determinism.

---

## On-chain Anchor
A minimal on-chain record that binds a schema hash (or proof root) to an addressable registry entry.

---

## Plugin
A component that supports a specific input type and produces IR. Plugins must be deterministic, versioned, and configurable.

---

## Proof
Verification material enabling independent integrity checks (e.g., Merkle root and inclusion proofs). Proofs bind the schema and manifest to verifiable hash roots.

---

## Publisher
The entity (typically a Solana pubkey) that registers a schema hash on-chain. Publishing does not imply truth; it provides an anchor and attribution.

---

## Registry
An optional Solana program that stores minimal records keyed by schema hash and supports version linking and discovery.

---

## Schema
A canonical structural definition emitted by SIGNIA. Schemas are versioned and hashed in a specific domain.

---

## Schema Hash
The canonical hash identifier of a schema computed from its canonical bytes in the schema hash domain.

---

## Structural Model
A graph-like representation of entities, edges, and constraints derived from an artifact, prior to canonicalization.

---

## Structure
The minimal, deterministic representation of relationships and constraints within an artifact:
entities, edges, types, and invariants. Structure excludes runtime execution.

---

## Trust Model
The assumptions about what must be trusted:
- inputs are not trusted
- outputs are verified by recomputation and proofs
- on-chain records are minimal and do not store large content
Trust is minimized by determinism and verifiability.

---

## Type
A structural constraint describing the shape of data (fields, kinds, allowable values). Types are part of structure.

---

## Version Link
A relationship between two schema hashes describing evolution (supersedes, compatible_with, forks_from). Stored off-chain or via registry metadata.

---

## Verification
The process of checking that a bundle is internally consistent:
- canonical bytes match stated hashes
- manifest references match schema/proof
- proof root matches leaf set and ordering rules

---

## Verifiability
The property that outputs can be validated independently without trusting the compiler runner.

---

## Workspace
A repository layout where multiple packages/crates are managed together. SIGNIA documentation may reference a monorepo workspace.

---
