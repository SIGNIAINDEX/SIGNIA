
# On-chain Instructions

This document specifies the Solana instruction set for the SIGNIA registry program. It includes instruction inputs, account requirements, authority rules, validation checks, and expected state transitions.

This document is designed to match an Anchor-based implementation, but the instruction semantics are independent of framework.

Related docs:
- `docs/onchain/registry-program.md`
- `docs/onchain/accounts.md`

---

## 1) Instruction summary

v1 instructions:

1. `InitializeConfig`
2. `RegisterSchema`
3. `UpdateSchemaMetadata`
4. `RevokeSchema`
5. `TransferPublisher` (optional but recommended)

Each instruction must be deterministic and have strict signer/authority constraints.

---

## 2) Common rules and conventions

### 2.1 Hash fields
All hash inputs are raw 32-byte arrays:
- `schema_hash: [u8; 32]`
- `proof_root: [u8; 32]`
- `manifest_hash: [u8; 32]`

Clients may encode as hex/base58, but on-chain inputs are bytes.

### 2.2 Optional fields (v1 encoding)
Optional hashes are encoded as:
- all zeros `[0u8; 32]` means absent
- any non-zero value means present

This avoids variable-length encoding and keeps fixed account size.

### 2.3 Status enum
- `0` = active
- `1` = revoked

### 2.4 Slot fields
Slots are recorded for indexing and auditing:
- `created_slot` set at record creation
- `updated_slot` set on any update/revoke/transfer

---

## 3) InitializeConfig

Creates the `RegistryConfig` PDA.

### 3.1 Inputs
- `admin: Pubkey`

### 3.2 Accounts
- `payer` — signer, writable
- `config` — PDA, writable (created)
- `system_program` — read-only

### 3.3 PDA derivation
- `config = PDA(["signia-registry-config"], program_id)`

### 3.4 Authority rules
- `payer` must sign.
- `admin` is stored in config.

### 3.5 Validation
- `config` must not already exist.
- `admin` must not be default pubkey (recommended check).
- PDA derivation must match.

### 3.6 State transition
Creates `RegistryConfig`:
- `admin = input admin`
- `version = 1`
- `flags = 0`
- `bump = derived bump`
- `reserved = zeroed`

---

## 4) RegisterSchema

Creates a `SchemaRecord` PDA for a schema hash.

### 4.1 Inputs
- `schema_hash: [u8; 32]`
- `proof_root: [u8; 32]` (optional; zero means absent)
- `manifest_hash: [u8; 32]` (optional; zero means absent)
- `flags: u32` (reserved; may be 0 in v1)

### 4.2 Accounts
- `payer` — signer, writable
- `publisher` — signer, read-only (authority recorded)
- `config` — PDA, read-only
- `record` — PDA derived from schema hash, writable (created)
- `system_program` — read-only

### 4.3 PDA derivation
- `record = PDA(["signia-schema", schema_hash], program_id)`

### 4.4 Authority rules
- `publisher` must sign.
- `payer` must sign (funds account creation).

### 4.5 Validation
- `config` must exist.
- `record` must not exist.
- `schema_hash` must not be all zeros (recommended).
- `record` PDA must match seeds and program ID.
- If `proof_root` is non-zero:
  - it must be 32 bytes (always true by type)
- If `manifest_hash` is non-zero:
  - it must be 32 bytes

Optional (recommended):
- reject `proof_root == schema_hash` only if your hash domains guarantee non-collision; generally unnecessary.
- enforce a policy that at least one of proof_root/manifest_hash must be present (optional).

### 4.6 State transition
Creates `SchemaRecord`:
- `schema_hash = input schema_hash`
- `proof_root = input proof_root`
- `manifest_hash = input manifest_hash`
- `publisher = publisher pubkey`
- `status = active`
- `created_slot = current slot`
- `updated_slot = current slot`
- `flags = input flags` (or 0 in v1)
- `version = 1`
- `bump = derived bump`
- `reserved = zeroed`

---

## 5) UpdateSchemaMetadata

Updates optional stored hashes or flags without changing identity.

### 5.1 Inputs
- `proof_root: [u8; 32]` (optional; zero means clear)
- `manifest_hash: [u8; 32]` (optional; zero means clear)
- `flags: u32` (reserved)

### 5.2 Accounts
- `authority` — signer, read-only
- `config` — PDA, read-only
- `record` — PDA, writable

### 5.3 Authority rules
Two policy variants:

Policy A (publisher-only):
- `authority` must equal `record.publisher`

Policy B (admin override):
- `authority` must equal `record.publisher` OR `config.admin`

This repository should document the chosen policy. v1 early-stage default is Policy B.

### 5.4 Validation
- `config` exists.
- `record` exists and owned by program.
- `record.status` may be active or revoked depending on policy:
  - recommended: allow updates only when active (revoked records should be immutable), except admin may update flags for indexing.
- authority check per selected policy.

### 5.5 State transition
Updates:
- `record.proof_root = input proof_root`
- `record.manifest_hash = input manifest_hash`
- `record.flags = input flags`
- `record.updated_slot = current slot`

---

## 6) RevokeSchema

Marks a schema record as revoked.

### 6.1 Inputs
- none (or `reason_code: u16` in future versions; not in v1)

### 6.2 Accounts
- `authority` — signer, read-only
- `config` — PDA, read-only
- `record` — PDA, writable

### 6.3 Authority rules
Same as UpdateSchemaMetadata:
- publisher-only or admin override policy

### 6.4 Validation
- record exists
- authority check passes
- if record already revoked:
  - either no-op (idempotent) or error
  - recommended: idempotent, but still update `updated_slot`

### 6.5 State transition
- `record.status = revoked`
- `record.updated_slot = current slot`

---

## 7) TransferPublisher (recommended)

Transfers publisher authority to a new pubkey.

### 7.1 Inputs
- `new_publisher: Pubkey`

### 7.2 Accounts
- `current_publisher` — signer, read-only
- `record` — PDA, writable

Optional (if enforcing config policy):
- `config` — PDA, read-only
- `admin` — signer (only if admin override transfer is allowed)

### 7.3 Authority rules
Default policy:
- only current publisher may transfer

Optional policy:
- admin can transfer in emergency (document if allowed)

### 7.4 Validation
- record exists
- record is active (recommended; prevent transfer on revoked)
- current_publisher matches record.publisher
- new_publisher is not default pubkey (recommended)
- if new_publisher equals current publisher:
  - allow no-op or error; recommended no-op

### 7.5 State transition
- `record.publisher = new_publisher`
- `record.updated_slot = current slot`

---

## 8) Error codes (recommended)

Use stable, explicit error codes for client UX:

- `ConfigAlreadyInitialized`
- `ConfigNotInitialized`
- `InvalidPda`
- `InvalidSchemaHash`
- `RecordAlreadyExists`
- `RecordNotFound`
- `Unauthorized`
- `InvalidStatus`
- `InvalidNewPublisher`

Errors should not include sensitive data.

---

## 9) Instruction data layout (Anchor guidance)

If using Anchor, instruction data is encoded via Borsh.

Recommended structs:

- `InitializeConfigArgs { admin: Pubkey }`
- `RegisterSchemaArgs { schema_hash: [u8; 32], proof_root: [u8; 32], manifest_hash: [u8; 32], flags: u32 }`
- `UpdateSchemaMetadataArgs { proof_root: [u8; 32], manifest_hash: [u8; 32], flags: u32 }`
- `TransferPublisherArgs { new_publisher: Pubkey }`

---

## 10) Client examples (conceptual)

### 10.1 Register a schema
Client flow:
1. Compute schema hash from canonical schema bytes.
2. Compute proof root from proof rules.
3. Derive record PDA using schema hash bytes.
4. Send `RegisterSchema` with:
   - schema_hash
   - proof_root (optional)
   - manifest_hash (optional)
5. Confirm transaction.

### 10.2 Verify a schema is anchored
Client flow:
1. Compute schema hash from local bundle.
2. Derive record PDA.
3. Fetch record and confirm:
   - record exists
   - status is active
   - stored hashes match local recomputation
   - publisher matches expected authority (optional)

---

## 11) Security notes

- Always validate PDA derivations.
- Enforce signer constraints strictly.
- Keep instruction set minimal.
- Prefer fixed-size inputs to avoid parsing ambiguity.
- Do not accept arbitrary string metadata in v1.

---

## 12) Related documents

- Account layouts: `docs/onchain/accounts.md`
- Registry overview: `docs/onchain/registry-program.md`
