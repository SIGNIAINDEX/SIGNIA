
# Error Codes

This document defines the error code taxonomy for the SIGNIA API and CLI. A consistent error model improves:
- client UX
- observability
- deterministic testing
- operational support

The API returns errors using `application/json` with a stable envelope:

```json
{
  "error": {
    "code": "SOME_CODE",
    "message": "Human readable message",
    "details": { "any": "json" }
  }
}
```

The CLI uses the same codes and prints a structured message.

Related docs:
- `docs/api/openapi.yaml`
- `docs/api/auth.md`
- `docs/determinism/determinism-contract.md`

---

## 1) Design principles

1. Codes are stable identifiers (SCREAMING_SNAKE_CASE).
2. Codes are not localized.
3. Messages are human-readable but not relied upon by clients.
4. `details` is optional and may include structured hints.
5. Do not leak secrets or absolute host paths in errors.
6. Deterministic errors: same input yields same code.

---

## 2) Top-level categories

Error codes are grouped by category prefix:

- `AUTH_*` — authentication and authorization
- `RATE_*` — rate limits and quotas
- `INPUT_*` — invalid or unsafe inputs
- `JOB_*` — job lifecycle and compilation pipeline
- `BUNDLE_*` — bundle storage, parsing, verification
- `SCHEMA_*` — schema lookup and decoding
- `ONCHAIN_*` — on-chain publishing and lookup
- `INTERNAL_*` — unexpected server failures
- `NOT_SUPPORTED_*` — feature flags / unsupported operations

Clients should treat unknown codes as fatal and display the message.

---

## 3) Authentication and authorization

### 3.1 Errors

- `AUTH_MISSING_API_KEY`
  - HTTP: 401
  - Meaning: Request is missing `X-API-Key`
  - Details: `{ "header": "X-API-Key" }`

- `AUTH_INVALID_API_KEY`
  - HTTP: 401
  - Meaning: Provided key is not recognized
  - Details: none

- `AUTH_FORBIDDEN`
  - HTTP: 403
  - Meaning: Authenticated but lacking permissions (scopes)
  - Details: `{ "requiredScopes": ["..."], "presentScopes": ["..."] }` (optional)

- `AUTH_TOKEN_EXPIRED` (if JWT enabled)
  - HTTP: 401
  - Meaning: Token expired
  - Details: `{ "exp": 1700000000 }`

---

## 4) Rate limits and quotas

- `RATE_LIMITED`
  - HTTP: 429
  - Meaning: Rate bucket exhausted
  - Details: `{ "policy": "verify", "retryAfterSeconds": 12 }`

- `RATE_CONCURRENCY_LIMITED`
  - HTTP: 429
  - Meaning: Too many in-flight operations
  - Details: `{ "policy": "compile", "inFlight": 4, "maxInFlight": 2 }`

- `RATE_QUOTA_EXCEEDED`
  - HTTP: 429
  - Protects: bytes/day, jobs/day, etc
  - Details: `{ "quota": "uploadBytesPerDay", "used": 123, "limit": 456 }`

---

## 5) Input validation and safety

- `INPUT_BAD_REQUEST`
  - HTTP: 400
  - Meaning: Malformed JSON or missing required fields
  - Details: `{ "field": "plugin.id" }` (optional)

- `INPUT_UNSUPPORTED_FORMAT`
  - HTTP: 400
  - Meaning: Input format not supported by selected plugin
  - Details: `{ "plugin": "openapi", "supported": ["yaml", "json"] }`

- `INPUT_TOO_LARGE`
  - HTTP: 413
  - Meaning: Uploaded body exceeds size limit
  - Details: `{ "maxBytes": 67108864 }`

- `INPUT_ARCHIVE_TRAVERSAL_DETECTED`
  - HTTP: 400
  - Meaning: Archive contains path traversal entries (`../` or absolute paths)
  - Details: `{ "entry": "../evil" }`

- `INPUT_SYMLINKS_DENIED`
  - HTTP: 400
  - Meaning: Input contains symlinks but policy denies them
  - Details: `{ "path": "artifact:/some/link" }`

- `INPUT_NETWORK_DISABLED`
  - HTTP: 400
  - Meaning: Job requests remote input but policy denies network
  - Details: `{ "policy": "deny" }`

- `INPUT_CHECKSUM_REQUIRED`
  - HTTP: 400
  - Meaning: Remote fetch requires pinned checksum or immutable reference
  - Details: `{ "uri": "https://..." }`

- `INPUT_CHECKSUM_MISMATCH`
  - HTTP: 400
  - Meaning: Remote or uploaded content hash mismatch
  - Details: `{ "expected": "...", "actual": "..." }`

---

## 6) Job lifecycle errors

- `JOB_NOT_FOUND`
  - HTTP: 404
  - Meaning: Job id not found

- `JOB_INVALID_STATE`
  - HTTP: 409
  - Meaning: Operation not allowed in current state
  - Details: `{ "state": "running", "allowed": ["created", "queued"] }`

- `JOB_MISSING_INPUT`
  - HTTP: 400
  - Meaning: Attempt to run job without any input
  - Details: none

- `JOB_CANCELED`
  - HTTP: 409
  - Meaning: Job was canceled and cannot be resumed
  - Details: none

- `JOB_FAILED`
  - HTTP: 409
  - Meaning: Job failed; inspect error details
  - Details: `{ "cause": "PLUGIN_PARSE_ERROR" }` (optional)

- `JOB_TIMEOUT`
  - HTTP: 409
  - Meaning: Job exceeded configured timeout
  - Details: `{ "timeoutMs": 300000 }`

- `JOB_LIMIT_EXCEEDED`
  - HTTP: 409
  - Meaning: Compilation exceeded resource limits
  - Details: `{ "limit": "maxNodes", "max": 200000, "observed": 250000 }`

---

## 7) Bundle errors

- `BUNDLE_NOT_FOUND`
  - HTTP: 404
  - Meaning: Bundle id not found

- `BUNDLE_INVALID_ARCHIVE`
  - HTTP: 400
  - Meaning: Uploaded bundle archive is invalid or missing required files
  - Details: `{ "missing": ["schema.json"] }`

- `BUNDLE_INVALID_SCHEMA`
  - HTTP: 400
  - Meaning: schema.json does not match Schema v1 or fails validation

- `BUNDLE_INVALID_MANIFEST`
  - HTTP: 400
  - Meaning: manifest.json fails validation

- `BUNDLE_INVALID_PROOF`
  - HTTP: 400
  - Meaning: proof.json fails validation

- `BUNDLE_HASH_MISMATCH`
  - HTTP: 400
  - Meaning: Recomputed hash does not match manifest/proof
  - Details: `{ "expected": "...", "actual": "...", "kind": "schema_hash" }`

- `BUNDLE_TAMPERED`
  - HTTP: 400
  - Meaning: Integrity checks failed (proof root mismatch, leaf mismatch, etc.)

- `BUNDLE_CANONICALIZATION_FAILED`
  - HTTP: 400
  - Meaning: Canonicalization produced invalid output
  - Details: `{ "reason": "invalid_json" }`

---

## 8) Schema lookup errors

- `SCHEMA_NOT_FOUND`
  - HTTP: 404
  - Meaning: No schema found for a given schema hash

- `SCHEMA_INVALID_HASH`
  - HTTP: 400
  - Meaning: Hash param is invalid format
  - Details: `{ "pattern": "^[0-9a-f]{64}$" }`

---

## 9) On-chain errors

- `ONCHAIN_NETWORK_UNAVAILABLE`
  - HTTP: 409
  - Meaning: RPC endpoint unreachable or degraded
  - Details: `{ "network": "mainnet-beta" }`

- `ONCHAIN_RECORD_NOT_FOUND`
  - HTTP: 404
  - Meaning: No on-chain SchemaRecord exists for schema hash

- `ONCHAIN_PUBLISH_FAILED`
  - HTTP: 409
  - Meaning: Publish transaction failed
  - Details: `{ "signature": "...", "logs": ["..."] }` (redact if needed)

- `ONCHAIN_POLICY_VIOLATION`
  - HTTP: 400
  - Meaning: Publish request violates configured policy (e.g., missing publisher)
  - Details: `{ "required": ["publisher"] }`

- `ONCHAIN_UNSUPPORTED_MODE`
  - HTTP: 400
  - Meaning: Requested publish mode not supported
  - Details: `{ "mode": "operator_sign" }`

---

## 10) Not supported / feature flags

- `NOT_SUPPORTED_FEATURE_DISABLED`
  - HTTP: 409
  - Meaning: Feature is disabled in server configuration
  - Details: `{ "feature": "onchain_publish" }`

- `NOT_SUPPORTED_PLUGIN`
  - HTTP: 400
  - Meaning: Plugin id unknown or not enabled
  - Details: `{ "plugin": "unknown" }`

---

## 11) Internal server errors

- `INTERNAL_ERROR`
  - HTTP: 500
  - Meaning: Unexpected exception; includes request id
  - Details: `{ "requestId": "..." }`

- `INTERNAL_STORAGE_ERROR`
  - HTTP: 500
  - Meaning: Storage layer failure
  - Details: `{ "backend": "sqlite" }`

- `INTERNAL_DEPENDENCY_ERROR`
  - HTTP: 500
  - Meaning: Dependency failed (e.g., hash library)
  - Details: `{ "dependency": "blake3" }`

---

## 12) Mapping to HTTP status codes

General mapping:
- 400: input problems and verification failures
- 401: missing/invalid auth
- 403: forbidden / lacking scope
- 404: missing resource
- 409: invalid state, conflicts, execution failures
- 413: payload too large
- 429: rate limits / quotas / concurrency
- 500: internal showing request id

---

## 13) Example error responses

### 13.1 Missing API key

```json
{
  "error": {
    "code": "AUTH_MISSING_API_KEY",
    "message": "Missing API key",
    "details": { "header": "X-API-Key" }
  }
}
```

### 13.2 Verification failure

```json
{
  "error": {
    "code": "BUNDLE_HASH_MISMATCH",
    "message": "Schema hash mismatch",
    "details": {
      "kind": "schema_hash",
      "expected": "0123...",
      "actual": "4567..."
    }
  }
}
```

---

## 14) CLI integration

The CLI should:
- exit non-zero for any error
- print the code first, then message
- optionally emit JSON via `--json`

Example:
- `SIGNIA_ERROR=BUNDLE_HASH_MISMATCH schema hash mismatch`

---

## 15) Related documents

- OpenAPI: `docs/api/openapi.yaml`
- Authentication: `docs/api/auth.md`
- Rate limits: `docs/api/rate-limits.md`
