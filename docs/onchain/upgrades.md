
# Upgrades

This document describes upgradeability strategy for the SIGNIA Solana registry program and its on-chain accounts. It covers:
- upgrade authority policy
- account versioning strategy
- reserved space usage
- migration patterns
- backward compatibility rules
- operational playbooks for upgrades

This is intended to be practical for real deployments and audits.

Related docs:
- `docs/onchain/accounts.md`
- `docs/onchain/instructions.md`
- `docs/onchain/pda-layout.md`
- `docs/security/security/supply-chain.md`

---

## 1) Goals

1. Keep the on-chain program small and auditable.
2. Enable additive changes without breaking clients.
3. Support controlled upgrades with explicit governance.
4. Ensure upgrades do not break determinism assumptions (registry is an anchor, not a compute engine).
5. Provide clear migration guidance when breaking changes are unavoidable.

---

## 2) Upgrade models on Solana

Common patterns:

### 2.1 Upgradeable BPF loader (typical)
- Program is deployed via the upgradeable loader.
- A single upgrade authority can deploy new program data.
- This supports iterative development but increases trust requirements.

### 2.2 Immutable program (no upgrades)
- Program is deployed as immutable.
- No upgrade authority remains.
- Highest trust for long-lived anchors, but no bugfixes.

### 2.3 Two-phase: upgradeable to immutable
- Start upgradeable during early iteration.
- After audits and stabilization, revoke upgrade authority and finalize as immutable.

Recommended for SIGNIA registry:
- Two-phase approach:
  - v1: upgradeable with strong controls
  - v1.1+ after audit: finalize as immutable or move to governance-controlled upgrades

---

## 3) Governance and authority policy

### 3.1 Roles
- `Program Upgrade Authority`: controls program upgrades via loader.
- `Registry Admin` (stored in RegistryConfig): controls certain on-chain actions depending on policy (admin override).
- `Publisher`: per-record authority for updates/revocation.

These roles are distinct.

### 3.2 Recommended authority setup
Early stage:
- Upgrade authority is a multisig (2/3 or 3/5).
- Registry admin is the same multisig or a separate governance key.
- Publishers are individual project keys or multisigs.

### 3.3 Emergency controls
If admin override policy is enabled:
- admin can revoke records in case of compromise
- admin actions should be transparent and monitored

---

## 4) Account versioning strategy

### 4.1 Version field
Every account includes a `version: u16`.

Rules:
- v1 layout: `version = 1`
- v2 layout: `version = 2`, etc.

Clients must:
- read the version field
- decode fields accordingly
- fail closed for unknown versions unless a compatibility layer exists

### 4.2 Reserved bytes
Accounts include reserved bytes to allow additive fields without changing account size.

Rules:
- reserved bytes can be repurposed only if documented
- repurposed fields must have stable encoding
- never shrink or reorder existing fields

---

## 5) Additive upgrades (preferred)

Additive upgrades do not require account migrations.

Examples:
- new flags in `flags` field
- additional status codes (if using flags + reserved bytes carefully)
- new optional hashes if reserved space exists
- indexing hints

Process:
1. Implement new logic in program code.
2. Update docs.
3. Add tests.
4. Deploy upgrade.
5. KeepP: clients that do not understand new fields still work because old fields remain stable.

---

## 6) Breaking upgrades and migration patterns

Breaking changes should be avoided, but if necessary:

### 6.1 New account type / new PDA prefix
Preferred breaking strategy:
- create new PDA prefixes for new records
- leave old records intact
- clients can support both versions

Example:
- v1 SchemaRecord: prefix `"signia-schema"`
- v2 SchemaRecord: prefix `"signia-schema-v2"`

Pros:
- no migrations required
- old anchors remain valid
Cons:
- clients must handle multiple versions

### 6.2 Migration with new accounts (recommended over resizing)
If migration is required:
- create new v2 accounts
- copy and transform data from v1
- mark v1 records as superseded via flags or optional pointers (if space exists)

Do not attempt in-place resizing; it is complex and risky.

### 6.3 Migration instruction
If you implement migration:
- define a dedicated instruction:
  - `MigrateSchemaRecordV1ToV2`
- require strong authority checks
- emit events/logs for indexing

---

## 7) Backward compatibility rules (normative)

- Never change PDA seeds for existing account types.
- Never change the meaning of existing fields.
- Never reorder fields in serialized layouts.
- Never repurpose non-reserved bytes.
- Additive changes must be opt-in for clients.
- Clients must fail closed on unknown account versions unless explicitly designed otherwise.

---

## 8) Operational playbook for upgrades

### 8.1 Pre-upgrade checklist
- audit changes (internal review at minimum)
- run full test suite
- verify account sizes and layout constants
- verify PDA derivation test vectors
- run static analysis and linting
- confirm no new nondeterministic behavior is introduced
- ensure docs and changelog are updated

### 8.2 Deployment steps
1. Deploy to devnet with the same seeds/prefixes.
2. Run integration tests and indexer checks.
3. Deploy to mainnet with multisig approval.
4. Monitor program logs and indexing.

### 8.3 Rollback strategy
If an upgrade is faulty:
- deploy a fix quickly
- if state is corrupted (rare in registry program):
  - consider emergency admin revocation of impacted records
  - provide public incident report

Note:
- on Solana, rollback is forward by deploying a new program version; old program data is not reinstated automatically.

---

## 9) Security considerations for upgrades

- Protect upgrade authority keys with multisig and hardware wallets.
- Restrict who can propose upgrades.
- Require reproducible builds and signed artifacts where possible.
- Pin toolchain versions (Anchor/Solana versions) for deterministic builds.
- Use CI with provenance (SLSA-like practices).

---

## 10) Recommended repository artifacts

To support upgrades safely, maintain:

- `programs/signia-registry/` source code
- `programs/signia-registry/tests/` integration tests
- `docs/onchain/accounts.md` with exact sizes
- `docs/onchain/pda-layout.md` with test vectors
- `CHANGELOG.md` with upgrade notes
- `SECURITY.md` for reporting vulnerabilities

---

## 11) Client upgrade guidance

Clients should:
- treat unknown versions as unsupported by default
- surface clear errors and links to docs
- support multiple PDA prefixes if v2 introduces new records
- provide explicit user controls for trust policy (admin override vs publisher-only)

---

## 12) Example: two-phase finalization

A practical plan:

Phase 1 (launch):
- deploy upgradeable program
- enable admin override for emergency response
- publish a roadmap for immutability

Phase 2 (stabilize):
- third-party audit
- reduce program surface area
- finalize program as immutable:
  - set upgrade authority to none (or a burn address)
  - keep admin override policy explicit and governed

Phase 3 (evolve):
- if new features require changes:
  - deploy a new program ID (v2 program), or
  - use governance-controlled upgrade authority with transparent processes

---

## 13) Related documents

- Accounts: `docs/onchain/accounts.md`
- Instructions: `docs/onchain/instructions.md`
- PDA layout: `docs/onchain/pda-layout.md`
- Registry program: `docs/onchain/registry-program.md`
- Supply chain security: `docs/security/security/supply-chain.md`
