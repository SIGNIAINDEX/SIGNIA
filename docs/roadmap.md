
# Roadmap

This roadmap outlines SIGNIA’s planned development in a way that is compatible with an open-source workflow. It focuses on reproducible milestones, deliverables, and verification-first progress.

Notes:
- Milestones are outcome-based, not time-based.
- Any change affecting determinism, hash domains, or bundle contracts must be versioned and documented.
- The on-chain registry remains minimal by design.

---

## Guiding principles

1. **Determinism first**  
   Every supported input type must compile into stable canonical bytes with reproducible hashes.

2. **Verifiability over convenience**  
   Features that improve trust and independent verification are prioritized.

3. **Minimal on-chain footprint**  
   On-chain records store identifiers and minimal metadata only.

4. **Composable structure**  
   Schemas should interoperate across plugins and be linkable by hash.

5. **Operational usability**  
   CI-friendly CLI and API contracts matter as much as internal architecture.

---

## Milestone 0: Repository and baseline quality gates

Deliverables:
- Monorepo layout with Rust crates, optional Solana program, and JS tooling
- CI workflows: Rust, Node, Solana program, E2E, CodeQL, Release, Docker
- Issue templates, PR template, security policy
- Basic documentation: overview, architecture, glossary, FAQ, roadmap

Acceptance criteria:
- CI passes on clean checkout
- `cargo fmt`, `cargo clippy`, `cargo test` succeed
- Node lint/build (if applicable) succeeds
- Docs build or lint (if applicable) succeeds

---

## Milestone 1: Core data model and canonical bundle v1

Goal:
Define the minimal stable contracts for bundles.

Deliverables:
- `schema.json` specification (v1)
- `manifest.json` specification (v1)
- `proof.json` specification (v1)
- Determinism rules document for v1:
  - canonical JSON encoding
  - ordering and normalization rules
  - domain-separated hashing
- JSON Schema files under `schemas/v1/` for validation

Acceptance criteria:
- A bundle can be validated via JSON Schema
- Hash computation is deterministic and tested via golden fixtures
- `signia verify` can verify bundles without network access

---

## Milestone 2: Compiler pipeline and verification engine

Goal:
Implement Parse → Infer → Canonicalize → Compile → Verify for v1.

Deliverables:
- IR v1 (internal, versioned)
- Canonicalization engine (stable ordering + canonical JSON)
- Hashing and proof construction (Merkle root)
- Verification engine:
  - schema hash verification
  - manifest linkage verification
  - proof root verification
- CLI commands:
  - `compile`
  - `verify`
  - `inspect` (human-readable summaries)

Acceptance criteria:
- Same input compiles to the same schema hash across runs
- Verification fails on any bundle tampering
- Golden fixtures cover key invariants and cross-platform normalization

---

## Milestone 3: Plugin system and first production-grade plugins

Goal:
Support real inputs with deterministic plugins.

Deliverables:
- Plugin interface and lifecycle (pure, deterministic contracts)
- Built-in plugins (initial set):
  - OpenAPI (YAML/JSON)
  - Config model (JSON/YAML)
  - Workflow graph (JSON)
- Plugin configuration versioning recorded in manifests
- Plugin-specific determinism notes and fixtures

Acceptance criteria:
- Each plugin has at least:
  - one minimal example
  - one realistic example
  - golden output fixtures
  - documented normalization rules
- Plugins do not make unpinned network calls by default

---

## Milestone 4: Store and distribution primitives

Goal:
Make bundles easy to cache, distribute, and retrieve.

Deliverables:
- Content-addressed local store:
  - store by schema hash and manifest hash
  - bundle import/export
- Bundle packaging formats:
  - directory bundle
  - archive bundle (optional)
- `signia fetch` for store retrieval
- Optional signature support (publisher identity) as non-breaking metadata

Acceptance criteria:
- Bundles are de-duplicated by hash
- Store supports offline verification workflows
- Bundle import/export preserves canonical bytes

---

## Milestone 5: Solana registry program v1 (minimal)

Goal:
Enable on-chain anchoring of schema identifiers.

Deliverables:
- Anchor-based registry program with:
  - register schema hash
  - optional minimal metadata
  - query patterns (via client)
  - basic version linking (optional)
- Rust client crate for registry interactions
- CLI `publish` and `fetch` (registry-aware)
- Localnet integration tests (Anchor test + CLI smoke tests)

Acceptance criteria:
- Registering a schema hash is deterministic and idempotent
- On-chain records do not store large content
- Registry client and CLI can round-trip:
  - compile → verify → publish → fetch

---

## Milestone 6: API service v1 (automation interface)

Goal:
Provide remote access for compilation and verification.

Deliverables:
- API endpoints (versioned):
  - compile
  - verify
  - store lookup
  - registry publish/fetch
- Job deduplication by input descriptor hash
- Structured error model
- OpenAPI specification for the API

Acceptance criteria:
- API endpoints are stable and versioned
- Stateless verification is supported
- E2E CI includes API smoke checks

---

## Milestone 7: SDKs and integration ergonomics

Goal:
Make SIGNIA easy to embed.

Deliverables:
- TypeScript SDK:
  - compile/verify requests
  - schema fetch and parsing
  - registry helpers
- Python SDK (optional, if needed)
- Type definitions aligned with schema v1 JSON schema
- Example integrations in `examples/`

Acceptance criteria:
- SDKs match canonical schema contracts
- SDKs include basic integration tests
- SDK docs include end-to-end examples

---

## Milestone 8: Console (operator UI) and structural exploration

Goal:
Provide a UI for inspecting and understanding structures.

Deliverables:
- Console UI:
  - schema viewer
  - manifest viewer
  - proof viewer
  - graph exploration (basic)
- “Console” assistant entrypoint (project bot) for guided usage
- Search integration (optional off-chain index)

Acceptance criteria:
- Console can load bundles from local store or API
- Console clearly separates hashed vs non-hashed metadata
- Console can display schema hash and verification status

---

## Milestone 9: Determinism hardening and compatibility policy

Goal:
Stabilize long-term guarantees.

Deliverables:
- Cross-platform determinism test suite
- Compatibility policy:
  - what is stable
  - what can evolve
  - how version bumps happen
- Migration guides for:
  - schema versions
  - plugin versions
  - registry upgrades (if any)

Acceptance criteria:
- Determinism regressions are caught by CI
- Bundle contracts have explicit version policy
- Registry upgrades do not break existing schema identifiers

---

## Milestone 10: Production operations and ecosystem integration

Goal:
Make it production-ready for continuous use.

Deliverables:
- Observability:
  - structured logs
  - metrics (compile duration, cache hit rate)
- Rate limiting and abuse protection for hosted API
- Optional off-chain indexer for search and analytics
- Ecosystem adapters (optional):
  - governance systems referencing schema hashes
  - CI attestations linking builds to schema identifiers

Acceptance criteria:
- Hosted deployments are reproducible (Docker)
- Operational runbooks exist
- Performance and resource usage are documented

---

## Backlog (not ordered)

- Additional plugins:
  - Git repository structural model (files/modules/deps)
  - Dataset formats (Parquet, CSV schema inference with strict rules)
  - Markdown/spec parsers with normative constraint extraction
  - AI workflow formats (tool graphs, agent configs)
- Signature and attestation support
- Multi-registry support (conceptual portability)
- Schema diff tooling:
  - structural diffs between schema hashes
  - compatibility checks
- Policy engine:
  - allowlists/denylists for plugin features
  - compile constraints for CI enforcement

---

## Contribution areas

Good first contributions:
- Documentation improvements
- New examples and fixtures
- Determinism tests and fixture tooling
- Plugin adapters for structured formats
- CLI UX improvements (without breaking output contracts)

Core maintainer review required:
- Hash domain changes
- Canonicalization rule changes
- Bundle contract changes
- Registry program account layout changes

---

## Governance and releases

Release practices (recommended):
- Tag releases with semantic versions (`vX.Y.Z`)
- Publish release artifacts (binaries, checksums)
- Maintain a changelog with compatibility notes
- Avoid breaking changes to bundle contracts without a major version bump

---

## Disclaimer

There is currently **no token issued**.

Any token claims are not part of this repository unless explicitly documented in official releases and channels.
