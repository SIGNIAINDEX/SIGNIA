
# Threat Model

This document describes the SIGNIA threat model: what we are protecting, what we assume, what can go wrong, and how the architecture mitigates risks.

SIGNIA is a structure-level compiler and optional Solana registry. The primary security posture is:
- **Inputs are untrusted**
- **Outputs are verifiable**
- **On-chain footprint is minimal**
- **Determinism is a security property**

---

## 1) Scope and security goals

### In scope
- Off-chain compiler pipeline (ingest, parse, infer, canonicalize, compile, verify)
- Plugin system and built-in plugins
- Bundle format (schema/manifest/proof) and verification rules
- Local store and bundle distribution
- API service (if enabled)
- Solana registry program (if enabled)
- CLI workflows in CI/CD

### Out of scope
- Truthfulness of external sources (SIGNIA verifies compilation integrity, not truth)
- End-user application security integrating SIGNIA (depends on integrator)
- Arbitrary content hosting and distribution networks
- Non-SIGNIA on-chain programs consuming schema hashes

### Security goals
1. **Integrity**: detect tampering of schemas, manifests, and proofs.
2. **Reproducibility**: allow independent recomputation and verification.
3. **Availability**: resist parser crashes and resource exhaustion.
4. **Isolation**: prevent untrusted inputs from escaping boundaries.
5. **Minimal trust**: avoid reliance on compiler operator honesty.
6. **On-chain safety**: prevent state corruption and unauthorized writes.

---

## 2) Assets we protect

### Off-chain assets
- Compiler process integrity and host environment
- Local store data integrity (content-addressed bundles)
- Deterministic rules and hashing domains
- Plugin execution safety and correctness
- API service availability and correctness

### On-chain assets
- Registry accounts and recorded schema identifiers
- Authority model (who can write or update)
- Instruction correctness and account constraints
- Anti-spam and rent-related considerations

### User/consumer assets
- Ability to verify bundles independently
- Correctness of schema hash references
- Stable interface contracts for integrations

---

## 3) Actors and trust assumptions

### Actors
- Publisher: compiles and publishes schema identifiers
- Verifier: independently verifies bundles
- Consumer: uses schema identifiers in applications or governance
- Attacker: provides malicious inputs or attempts to poison records

### Trust assumptions
- Compiler operator is not trusted for correctness; verification must stand alone.
- Solana network provides consensus integrity (standard chain assumption).
- Cryptographic primitives used for hashing are secure under standard assumptions.
- Build toolchains and dependencies can be attacked; mitigate via supply-chain practices.

---

## 4) Entry points and attack surfaces

### Compiler / CLI
- Local files and directories
- Repository archives (tar/zip)
- Remote sources (if enabled)
- Plugin configuration and CLI flags
- Output bundle writing and store import/export

### API service
- HTTP endpoints for compile/verify/store operations
- Authentication/authorization (if deployed)
- Job queue and caching layers
- Request payload sizes and rate limiting

### Solana registry program
- Instructions and accounts
- PDA derivation and seed handling
- Account sizing and serialization
- Upgrade authority and governance

### CI/CD and release pipeline
- GitHub Actions workflows
- Dependency installation
- Artifact publishing (binaries, containers)

---

## 5) Threat categories (STRIDE)

### Spoofing
- Impersonating a publisher identity
- Forging registry records or signatures

Mitigations:
- On-chain publisher is a pubkey (signature required)
- Optional off-chain signatures can bind publisher identity to a bundle
- Clients should treat publisher identity as metadata unless policy-enforced

### Tampering
- Modifying `schema.json`, `manifest.json`, or `proof.json`
- Replacing a bundle with different content but same name
- Manipulating plugin outputs

Mitigations:
- Content-addressed identifiers (schema hash, proof root)
- `verify` recomputes from canonical bytes
- Domain-separated hashing prevents cross-context substitution
- Golden fixtures and CI protect determinism rules

### Repudiation
- Publisher denies having published a schema hash
- Operator denies having compiled a bundle

Mitigations:
- On-chain transaction history is non-repudiable
- Optional signed bundle attestations (future extension)
- Manifests record tool versions and pinned inputs

### Information disclosure
- Secrets leaked via compilation (tokens, private paths)
- Private inputs included in hashed outputs unintentionally

Mitigations:
- Explicit prohibition on embedding secrets in bundles
- Path normalization removes host-specific absolute paths
- API service should redact sensitive headers and payload fields
- CI checks should detect accidental secret inclusion

### Denial of service
- Zip bombs, large files, deeply nested structures
- Parser worst-case complexity
- Cyclic graphs or pathological IR causing exponential behavior
- API request floods

Mitigations:
- Hard size limits on inputs and per-file processing
- Timeouts per stage and per plugin
- Streaming parsing where possible
- Bounded graph size with clear failure modes
- API rate limiting and request size limits

### Elevation of privilege
- Path traversal to read or overwrite files outside workspace
- Symlink tricks escaping the intended input root
- Command injection via plugin configuration
- RCE via unsafe parsers

Mitigations:
- Strict input sandbox: root mapping and deny-by-default filesystem access outside root
- Symlink policy (deny or strict resolve) and canonical path validation
- No shell invocation from plugins (direct library calls only)
- Use memory-safe languages and vetted parsers

---

## 6) Threat scenarios and mitigations

### Scenario A: Malicious repository triggers parser crash
Attack:
- Crafted files exploit parser bugs or worst-case behavior.

Mitigations:
- Use robust parsers and fuzzing for plugins
- Limits: max file size, max depth, max nodes/edges
- Fail closed with actionable errors
- Isolate parsing in a constrained process where feasible

### Scenario B: Nondeterminism causes hash drift
Attack:
- Output changes across runs without input changes, breaking trust.

Mitigations:
- Determinism rules are explicit and versioned
- Canonical JSON encoding
- Stable ordering for every collection
- Golden fixtures in CI
- Ban timestamps, randomness, locale/time in hashed domains

### Scenario C: Bundle tampering during distribution
Attack:
- Adversary modifies files in a bundle, then redistributes.

Mitigations:
- `verify` recomputes schema hash and proof root
- Store uses content addressing; tampering changes hashes
- Consumers must verify before trusting

### Scenario D: Registry poisoning with misleading metadata
Attack:
- Publish a schema hash with misleading labels.

Mitigations:
- Registry stores minimal metadata; clients treat metadata as untrusted
- Consumers should rely on verification and policy allowlists for publishers
- Optional indexing layer can add curated metadata off-chain

### Scenario E: Remote input substitution
Attack:
- A URL changes content after compilation; others cannot reproduce.

Mitigations:
- Prefer pinned refs (commit SHA, checksum)
- Network access default deny
- If allowed, require content addressing or immutable ref verification

### Scenario F: API abuse and resource exhaustion
Attack:
- Flood compile endpoints or large payloads.

Mitigations:
- AuthN/AuthZ (deployment decision)
- Rate limits, quotas, and per-job limits
- Deduplication by input descriptor hash
- Backpressure and queue limits

### Scenario G: On-chain account corruption or unauthorized updates
Attack:
- Use wrong PDA seeds, bypass constraints, overwrite data.

Mitigations:
- PDA derivation uses schema hash seeds and program-owned accounts only
- Anchor constraints enforce ownership and signer requirements
- Instruction handlers validate record existence and idempotency
- Optional governance around upgrade authority

---

## 7) Determinism as a security property

Determinism reduces attack surface:
- Removes ambiguity about what a hash represents
- Allows any party to recompute and audit outputs
- Prevents hidden state or time-based manipulation
- Enables reproducible CI validation

Determinism failures are treated as security issues when they could:
- allow substitution attacks
- cause verification bypass
- mislead consumers about schema identity

---

## 8) Supply-chain and CI security

Risks:
- Dependency compromise
- Malicious build scripts
- CI runner compromises
- Artifact replacement

Mitigations:
- Lockfiles (`Cargo.lock`, `pnpm-lock.yaml`)
- `cargo-deny` and `cargo-audit`
- CodeQL scanning for Rust and JS/TS
- Release artifacts include checksums
- Prefer pinned toolchain versions for reproducibility

Recommended operational practices:
- Require PR reviews and branch protection
- Use signed tags/releases if possible
- Restrict GitHub Actions permissions (least privilege)

---

## 9) Logging and sensitive data handling

Rules:
- Never log secrets (tokens, keys, credentials)
- Avoid logging full inputs when they may contain private data
- Prefer structured logs with redaction hooks
- Store only what is needed for reproducibility

API service recommendations:
- Use request IDs
- Redact headers by default
- Enforce max body size
- Avoid storing raw payloads unless explicitly configured

---

## 10) Security testing strategy

Recommended practices:
- Fuzz parsers and canonicalization routines
- Property tests for determinism:
  - run compile twice; compare bytes/hashes
- Negative tests for verification:
  - mutate schema/manifest/proof; ensure verify fails
- Load tests for DoS resilience
- Static analysis and dependency audits in CI

---

## 11) Incident response

If you suspect a vulnerability:
- Do not open a public issue with exploit details.
- Follow `SECURITY.md` for responsible disclosure.

For determinism regressions:
- Treat as high priority.
- Pin affected versions and document the regression and mitigation.

---

## 12) Security non-goals

- Preventing publication of “bad” structures on-chain
- Ensuring external artifacts are truthful or safe to execute
- Providing privacy guarantees for public inputs
- Replacing application-level security decisions

SIGNIA provides verifiable structure. Consumers must still apply policy.

---

## Appendix: Checklist for security-sensitive changes

Changes that require extra scrutiny:
- Hash domain changes
- Canonical JSON encoding changes
- Ordering rule changes
- Plugin sandbox and file access changes
- Registry program account layout or instruction changes
- Allowing network access in plugins

Minimum requirements:
- Update specifications and versions
- Add or update golden fixtures
- Add negative verification tests
- Document migration and compatibility impact
