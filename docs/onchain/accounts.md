
# On-chain Accounts

This document describes the on-chain account model for SIGNIA’s Solana registry program. It provides account layouts, sizing guidance, PDA derivations, and client-side decoding expectations.

This is a specification document for the account data and is intended to match the actual program implementation in:
- `programs/signia-registry/` (or equivalent)

Related docs:
- `docs/onchain/registry-program.md`
- `docs/determinism/hashing.md`
- `docs/security/security/threat-model.md`

---

## 1) Account inventory

SIGNIA registry program accounts:

1. `RegistryConfig` (singleton)
2. `SchemaRecord` (one per schema hash)

Optional future accounts (not required for v1):
- `PublisherProfile`
- `NamespaceRecord`
- `IndexRecord`

v1 focuses on minimal surface area.

---

## 2) PDA derivation

### 2.1 Seeds (normative)

`RegistryConfig` PDA:
- seeds: `["signia-registry-config"]`
- bump: derived by Solana runtime

`SchemaRecord` PDA:
- seeds: `["signia-schema", schema_hash_bytes]`
- bump: derived by Solana runtime

Rules:
- seeds are ASCII bytes, constant and documented
- schema hash is raw 32 bytes, not string-encoded
- do not use variable-length or user-controlled strings as seeds

### 2.2 Client derivation examples (conceptual)

Clients compute:
- `config_pda = find_program_address(["signia-registry-config"], program_id)`
- `record_pda = find_program_address(["signia-schema", schema_hash_bytes], program_id)`

---

## 3) Serialization and discriminators

### 3.1 Anchor discriminator (if Anchor is used)
If implemented with Anchor:
- each account begins with an 8-byte discriminator
- account sizes must include the discriminator

If not using Anchor:
- define your own version byte(s) and type tags.

This spec assumes Anchor-style discriminators for clarity, but the fields are the same either way.

---

## 4) RegistryConfig account

### 4.1 Purpose
Stores:
- admin authority
- program config flags
- versioning and reserved space

### 4.2 Field layout (v1)

| Field | Type | Bytes | Notes |
|------|------|------:|------|
| discriminator | [u8; 8] | 8 | Anchor discriminator |
| admin | Pubkey | 32 | upgrade / emergency authority |
| bump | u8 | 1 | PDA bump |
| version | u16 | 2 | schema for account layout |
| flags | u32 | 4 | reserved feature flags |
| reserved | [u8; 64] | 64 | future upgrades |

Total (v1, Anchor): 8 + 32 + 1 + 2 + 4 + 64 = 111 bytes

Round up for alignment and future proofing when allocating (Anchor often requires exact size; keep stable).

Recommended allocation size:
- 128 bytes (plus discriminator already included in 111)

If using Anchor, set:
- `space = 8 + 32 + 1 + 2 + 4 + 64`

### 4.3 Invariants
- `admin` is a valid pubkey
- `version == 1` for v1
- bump matches PDA derivation

---

## 5) SchemaRecord account

### 5.1 Purpose
Stores integrity anchors and publisher identity for a schema hash.

### 5.2 Field layout (v1)

| Field | Type | Bytes | Notes |
|------|------|------:|------|
| discriminator | [u8; 8] | 8 | Anchor discriminator |
| schema_hash | [u8; 32] | 32 | required |
| proof_root | [u8; 32] | 32 | optional; all zeros means absent |
| manifest_hash | [u8; 32] | 32 | optional; all zeros means absent |
| publisher | Pubkey | 32 | authority for updates |
| created_slot | u64 | 8 | slot at registration |
| updated_slot | u64 | 8 | slot at last update |
| status | u8 | 1 | 0=active, 1=revoked |
| bump | u8 | 1 | PDA bump |
| version | u16 | 2 | account layout version |
| flags | u32 | 4 | reserved flags |
| reserved | [u8; 64] | 64 | future upgrades |

Total (v1, Anchor):
8 + 32 + 32 + 32 + 32 + 8 + 8 + 1 + 1 + 2 + 4 + 64 = 224 bytes

Recommended allocation size:
- exactly `224` if using Anchor with fixed fields
- optionally increase reserved bytes (e.g., 128) if you want more future space, but keep stable after deployment

### 5.3 Optional hash semantics
`proof_root`:
- if absent: all zeros `[0u8; 32]`
- if present: must be a valid computed proof root

`manifest_hash`:
- if absent: all zeros
- if present: must match the manifest hash computed from hashed view

This avoids variable-sized `Option<[u8; 32]>` encoding complexity.

### 5.4 Status semantics
- `status = 0`: active
- `status = 1`: revoked

Revocation means:
- record remains addressable and immutable in identity
- clients should treat revoked anchors as not valid for trust

### 5.5 Invariants
- `schema_hash` must match PDA seed bytes
- `version == 1` for v1
- bump matches PDA derivation
- `created_slot <= updated_slot`
- status is in allowed enum range

---

## 6) Rent and sizing guidance

### 6.1 Why size matters
Solana accounts are allocated at creation. Over-allocation increases rent, under-allocation prevents upgrades.

### 6.2 Recommended sizing policy
- Use a fixed size per account type.
- Include reserved bytes to allow non-breaking upgrades.
- Keep reserved space stable once deployed.

Suggested reserved sizes:
- Config: 64 bytes is sufficient for most early upgrades.
- Record: 64 bytes minimum; 128 if you anticipate expansions.

---

## 7) Client decoding

### 7.1 Canonical encoding
Clients should treat stored hashes as raw bytes:
- do not assume base58 or hex in account storage

When displaying:
- show hex or base58 as a derived representation
- ensure consistent formatting

### 7.2 Verification logic (client side)
Clients should:
1. Compute schema hash from `schema.json` canonical bytes.
2. Derive record PDA using raw schema hash bytes.
3. Fetch record and verify:
   - record.schema_hash == computed
   - record.status == active
4. If proof root or manifest hash are present on-chain:
   - recompute and compare to stored values.

Clients should fail closed on mismatches.

---

## 8) Upgrade strategy

### 8.1 Version field
The `version` field supports future layout changes.

Rules:
- v1: version = 1
- If a breaking layout change is introduced:
  - version increments
  - new instructions or migration path is defined
  - existing accounts remain readable

### 8.2 Reserved bytes
Reserved bytes may be used for:
- additional status enums
- timestamps
- signatures
- indexing hints
- namespace bindings

Any use must be documented and tested.

---

## 9) Security notes

- Do not store user-provided arbitrary strings on-chain.
- Keep seed derivations strict and collision-resistant.
- Enforce signer constraints in instructions.
- Avoid dynamic account resizing.
- Keep program code minimal and audited.

---

## 10) Appendix: Example Anchor space constants

If using Anchor, you can define:

- `RegistryConfig::LEN = 8 + 32 + 1 + 2 + 4 + 64`
- `SchemaRecord::LEN = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 1 + 1 + 2 + 4 + 64`

Use these constants when allocating.

---

## 11) Related documents

- Registry program spec: `docs/onchain/registry-program.md`
- Hashing: `docs/determinism/hashing.md`
- Threat model: `docs/security/security/threat-model.md`
