
# Usage

This document explains how to use the SIGNIA CLI in practical workflows. It focuses on real tasks:
- compile inputs into a bundle
- verify bundles deterministically
- inspect schema, manifest, and proof
- work with plugins
- publish anchors on-chain (optional)
- integrate in CI

Related docs:
- `docs/cli/installation.md`
- `docs/plugins/writing-a-plugin.md`
- `docs/determinism/determinism-contract.md`
- `docs/onchain/registry-program.md`

---

## 1) CLI overview

The SIGNIA CLI provides four core workflows:

1. Compile
- input → IR → schema/manifest/proof bundle

2. Verify
- bundle → recompute hashes and proof → verify integrity

3. Inspect
- summarize schema/manifest/proof
- extract hashes and metadata

4. On-chain (optional)
- publish anchors for schema hash and proof root
- look up anchors

---

## 2) Quick examples

### 2.1 Compile a directory
```bash
signia compile --plugin repo --input ./my-project --out ./out --safe
```

Outputs:
- `out/schema.json`
- `out/manifest.json`
- `out/proof.json`

### 2.2 Verify a bundle
```bash
signia verify --bundle ./out
```

### 2.3 Print the schema hash
```bash
signia inspect bundle --bundle ./out --json | jq -r .schemaHash
```

---

## 3) Compile

### 3.1 Basic compile
```bash
signia compile \
  --plugin openapi \
  --input ./specs/petstore.yaml \
  --out ./out \
  --safe
```

### 3.2 Plugin selection
`--plugin` chooses the structure extractor. Examples:
- `repo` — repository filesystem structure
- `openapi` — OpenAPI v3 documents
- `dataset` — dataset structure and manifest (optional)

Available plugins:
```bash
signia plugins list
```

Show plugin details:
```bash
signia plugins info openapi
```

### 3.3 Plugin configuration
Plugin config can be passed as JSON:

```bash
signia compile \
  --plugin repo \
  --input ./my-project \
  --out ./out \
  --plugin-config ./configs/repo.json \
  --safe
```

Example config file:
```json
{
  "include_globs": ["**/*"],
  "exclude_globs": ["**/.git/**", "**/target/**"],
  "emit_digests": true,
  "max_file_bytes": 1048576
}
```

The canonicalized plugin config hash is recorded in the manifest.

### 3.4 Output modes
Write outputs to:
- a directory (`--out ./out`)
- a zip archive (`--out ./out.zip`, if supported)

Example:
```bash
signia compile --plugin openapi --input ./spec.yaml --out ./out.zip --safe
```

### 3.5 Determinism flags
`--safe` enables conservative defaults:
- deny network
- deny symlinks (or resolve-within-root if configured)
- enforce canonical newlines and UTF-8
- apply strict resource limits

You can override policies explicitly (if supported by build):
```bash
signia compile --plugin repo --input ./x --out ./out \
  --policy-network deny \
  --policy-symlinks deny \
  --limit-max-files 20000 \
  --limit-timeout-ms 300000
```

### 3.6 Remote inputs (optional)
If the CLI supports pinned remote fetch:
```bash
signia compile \
  --plugin repo \
  --input-url https://example.com/repo.tar.gz \
  --input-sha256 <hex> \
  --out ./out \
  --safe
```

Pinned checksum is required by default.

---

## 4) Verify

Verification ensures:
- schema canonicalization and schema hash match
- proof root recomputation matches
- manifest links to schema and proof correctly

### 4.1 Verify a directory bundle
```bash
signia verify --bundle ./out
```

### 4.2 Verify a zip bundle
```bash
signia verify --bundle ./out.zip
```

### 4.3 Strict vs non-strict mode
Strict mode fails on warnings:
```bash
signia verify --bundle ./out --strict
```

Non-strict mode reports warnings but exits 0 (optional):
```bash
signia verify --bundle ./out --no-strict
```

### 4.4 Tamper checks
If any file is modified:
- verification must fail with a stable error code
- the CLI should print the failing check

Example output (conceptual):
- `BUNDLE_HASH_MISMATCH schema hash mismatch`

---

## 5) Inspect

### 5.1 Inspect a bundle
```bash
signia inspect bundle --bundle ./out
```

### 5.2 Output JSON for tooling
```bash
signia inspect bundle --bundle ./out --json
```

Example JSON fields:
- `schemaHash`
- `proofRoot`
- `manifestHash`
- `artifact.kind`
- `artifact.name`
- `counts.entities`, `counts.edges`

### 5.3 Inspect schema contents
```bash
signia inspect schema --bundle ./out
```

Optional filters (if supported):
- `--kind entity`
- `--id ent:file:src/main.rs`

### 5.4 Inspect proof
```bash
signia inspect proof --bundle ./out
```

Common fields:
- `root`
- `leafCount`
- `hashAlg`
- `domain`

---

## 6) Plugins

### 6.1 List plugins
```bash
signia plugins list
```

### 6.2 Plugin info
```bash
signia plugins info repo
```

### 6.3 Run a plugin-only IR dump (optional)
Some builds may support dumping IR:
```bash
signia compile --plugin repo --input ./x --out ./out --emit-ir
```

This is useful for debugging plugins.

---

## 7) On-chain operations (optional)

If the CLI includes on-chain support, it can publish and query anchors.

### 7.1 Publish a schema anchor
```bash
signia onchain publish \
  --bundle ./out \
  --network devnet \
  --program-id <REGISTRY_PROGRAM_ID> \
  --payer-keypair ~/.config/solana/id.json
```

This submits a transaction calling `RegisterSchema` to create a SchemaRecord PDA.

### 7.2 Lookup a record
```bash
signia onchain get \
  --schema-hash <hex> \
  --network devnet \
  --program-id <REGISTRY_PROGRAM_ID>
```

### 7.3 Policy notes
Publishing may require:
- publisher signature
- admin override rules

See:
- `docs/onchain/instructions.md`

---

## 8) CI integration

### 8.1 Determinism test in CI
In CI, compile the same fixture twice and compare outputs:

```bash
rm -rf out1 out2
signia compile --plugin openapi --input fixtures/openapi/petstore --out out1 --safe
signia compile --plugin openapi --input fixtures/openapi/petstore --out out2 --safe
diff -r out1 out2
signia verify --bundle out1
```

### 8.2 Bundle verification gate
If you accept third-party bundles:
- verify bundle before using it

```bash
signia verify --bundle ./incoming-bundle
```

---

## 9) Troubleshooting

### 9.1 Common errors

- `AUTH_MISSING_API_KEY` (API mode only)
- `INPUT_TOO_LARGE`
- `JOB_LIMIT_EXCEEDED`
- `BUNDLE_HASH_MISMATCH`

See:
- `docs/api/error-codes.md`

### 9.2 Non-deterministic outputs
If you see drift:
- ensure input bytes are identical
- ensure plugin config is identical
- ensure no network fetch occurs
- prefer Docker for consistency

---

## 10) Example workflows

### 10.1 Compile and publish (end-to-end)
1. Compile:
```bash
signia compile --plugin repo --input ./repo --out ./out --safe
```

2. Verify:
```bash
signia verify --bundle ./out
```

3. Publish:
```bash
signia onchain publish --bundle ./out --network devnet --program-id <ID> --payer-keypair ~/.config/solana/id.json
```

4. Share schema hash:
- others can fetch bundle and verify against on-chain record

### 10.2 Third-party verification
- Download bundle
- Verify
- Lookup record
- Compare hashes

---

## 11) Related documents

- Installation: `docs/cli/installation.md`
- Plugin authoring: `docs/plugins/writing-a-plugin.md`
- On-chain registry: `docs/onchain/registry-program.md`
- Determinism: `docs/determinism/determinism-contract.md`
