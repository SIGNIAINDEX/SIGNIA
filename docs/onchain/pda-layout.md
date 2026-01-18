
# PDA Layout

This document defines the program-derived address (PDA) layout for the SIGNIA Solana registry program. It specifies seeds, derivation rules, address namespaces, and recommended client-side helpers.

This doc is normative for v1:
- changing seeds breaks address derivation and client integrations
- any future expansion must use new seed prefixes rather than reusing existing ones

Related docs:
- `docs/onchain/registry-program.md`
- `docs/onchain/accounts.md`
- `docs/onchain/instructions.md`

---

## 1) Overview

The registry program uses PDAs to create stable, deterministic addresses for:
- global configuration
- per-schema records

PDAs ensure that:
- records are uniquely keyed by schema hash
- no private keys are required for program-owned accounts
- clients can derive addresses locally

---

## 2) Seed prefixes (v1)

All seed prefixes are ASCII byte strings.

v1 prefixes:

1. Config
- prefix: `"signia-registry-config"`

2. Schema record
- prefix: `"signia-schema"`

Rules:
- prefixes MUST remain constant for v1 deployments
- prefixes MUST be ASCII and fixed-length
- do not use user-controlled strings as prefixes

---

## 3) PDA derivations (v1)

### 3.1 RegistryConfig PDA
Seeds:
- `["signia-registry-config"]`

Derivation:
- `config_pda = find_program_address([b"signia-registry-config"], program_id)`

### 3.2 SchemaRecord PDA
Seeds:
- `["signia-schema", schema_hash_bytes]`

Where:
- `schema_hash_bytes` is the raw 32-byte schema hash computed from canonical `schema.json` bytes.

Derivation:
- `record_pda = find_program_address([b"signia-schema", schema_hash_bytes], program_id)`

---

## 4) Seed encoding rules

### 4.1 Schema hash seed encoding
The schema hash MUST be used as raw bytes:
- NOT hex string bytes
- NOT base58 string bytes
- NOT JSON string bytes

Clients must decode the schema hash into 32 bytes before derivation.

### 4.2 Fixed-length constraints
Solana PDA seeds have a per-seed length limit (32 bytes) and a total length limit.

This layout uses:
- 1 seed of ~21 bytes (prefix)
- 1 seed of 32 bytes (schema hash)

This fits within constraints.

---

## 5) Namespace expansion strategy

Future expansions should add new prefixes.

Examples (future, not in v1):
- `"signia-publisher"` + publisher pubkey
- `"signia-namespace"` + namespace hash
- `"signia-index"` + index key

Never change the meaning of existing prefixes in-place.

---

## 6) Collision and uniqueness

Because schema hashes are content-addressed and 32 bytes:
- collision probability is negligible with cryptographic hash functions
- PDA uniqueness is effectively guaranteed for practical purposes

However:
- ensure the schema hash function is stable and domain-separated per `docs/determinism/hashing.md`

---

## 7) Client-side helper recommendations

### 7.1 Provide a small helper library
Publish a client helper package that exposes:
- `deriveConfigPda(programId)`
- `deriveSchemaRecordPda(programId, schemaHashBytes)`

Avoid exposing raw seed strings in many places; keep them centralized.

### 7.2 Validation checks
Clients should validate:
- schema hash length is exactly 32
- schema hash is not all zeros (optional, but recommended)
- derived PDA matches expected addresses for test vectors

---

## 8) Test vectors (recommended)

Maintain test vectors in the repo:
- fixed program ID
- fixed schema hash bytes
- expected record PDA
- expected config PDA

Place under:
- `packages/client/test-vectors.json`
- `programs/signia-registry/tests/test_vectors.rs`

This ensures all clients derive the same addresses.

Example (illustrative values only):

```json
{
  "program_id": "SIGNiA111111111111111111111111111111111111",
  "schema_hash_hex": "0123012301230123012301230123012301230123012301230123012301230123",
  "expected_config_pda": "CfgPdaBase58...",
  "expected_record_pda": "RecPdaBase58..."
}
```

---

## 9) Security notes

- PDA seeds are public; do not include secrets.
- Do not include user-controlled strings as seeds.
- Ensure schema hash bytes are validated.
- Always verify account ownership on-chain (program ID matches).

---

## 10) Related documents

- Accounts: `docs/onchain/accounts.md`
- Instructions: `docs/onchain/instructions.md`
- Registry program: `docs/onchain/registry-program.md`
- Hashing: `docs/determinism/hashing.md`
