
# Authentication

This document defines authentication and authorization for the SIGNIA API service.

The API is designed to be self-hosted. Authentication is intentionally simple and predictable so it works in:
- local development
- private deployments
- production behind an API gateway

The default auth mechanism is an API key in a request header, with optional extensions for JWT/OIDC and request signing.

---

## 1) Goals

1. Prevent unauthorized access to compilation, verification, and bundle storage endpoints.
2. Support multiple tenants / keys (optional).
3. Keep auth deterministic and easy to audit.
4. Provide operational guidance for key rotation and rate limiting.

---

## 2) Default scheme: API key

### 2.1 Header
Clients send:

- `X-API-Key: <key>`

The OpenAPI spec defines this under `components.securitySchemes.ApiKeyAuth`.

### 2.2 Key format
API keys SHOULD be:
- random (128+ bits entropy)
- URL-safe
- non-guessable

Recommended format:
- `sk_signia_<base64url 32 bytes>`

Example:
- `sk_signia_3YlqJq7Qk1c9bEJ9eRk9a1Wm2vP9q0VtQh0iU3o0Jm8`

Keys must be treated like secrets:
- do not log them
- do not embed in client-side code

### 2.3 Server-side validation
The service validates the key by:
- exact match against configured keys
- constant-time comparison recommended

Key store options:
- environment variables (dev)
- a local config file (private deployments)
- a database or secret manager (production)

---

## 3) Authorization model

### 3.1 Default: global access per key
Each key grants access to the API.

Optionally associate keys with:
- a tenant id
- a role
- a set of permissions (scopes)

### 3.2 Suggested scopes (optional)
If you implement scopes:
- `jobs:create`
- `jobs:read`
- `jobs:cancel`
- `bundles:upload`
- `bundles:read`
- `bundles:verify`
- `schemas:read`
- `onchain:publish`
- `onchain:read`

This can be implemented without changing the API shape by checking per-route policy.

---

## 4) Rate limiting (recommended)

Rate limiting is not authentication, but it prevents abuse.

Recommended limits:
- job creation: low (e.g., 10/min per key)
- input uploads: size-limited
- verification: moderate (e.g., 60/min per key)
- bundle downloads: moderate

Rate limiting strategies:
- token bucket per key
- IP + key combined
- burst allowance + steady rate

Return:
- `429 Too Many Requests`
- include `Retry-After` header (recommended)

---

## 5) Key rotation

### 5.1 Rotation policy
Production deployments should support:
- multiple active keys at once
- staged rotation

Process:
1. Generate a new key.
2. Add it to server key store.
3. Update clients to use new key.
4. Revoke the old key after a grace period.

### 5.2 Emergency rotation
If a key is leaked:
- revoke immediately
- audit server logs and request traces
- rotate any associated credentials

---

## 6) Secure transport

Authentication must be used over TLS:
- HTTPS only in production.

If you run behind a proxy/gateway:
- terminate TLS at the gateway
- ensure gateway-to-service traffic is protected (mTLS recommended) or within a private network

---

## 7) Logging and privacy

Do:
- log request ids, route names, status codes, durations
- log job ids and bundle ids
- redact or omit sensitive request bodies

Do not:
- log API keys
- log uploaded artifacts
- log raw schema bytes

---

## 8) Optional extension: JWT / OIDC

For enterprise environments, you can add JWT:
- `Authorization: Bearer <token>`

Rules:
- verify signature against JWKs
- validate issuer, audience, expiration
- map claims to scopes

This is optional and should not be required for baseline deployments.

---

## 9) Optional extension: request signing

For high-trust environments and on-chain related operations, consider request signing:
- client signs request body hash with an Ed25519 key (Solana-compatible)
- server verifies signature and uses pubkey as identity

Suggested headers:
- `X-SIGNIA-PUBKEY: <base58 pubkey>`
- `X-SIGNIA-SIGNATURE: <base64 signature>`
- `X-SIGNIA-TIMESTAMP: <unix seconds>`

Rules:
- signature over: `method || path || timestamp || body_sha256`
- enforce small timestamp skew window (e.g., 60s)
- reject replay via nonce cache (optional)

This mode is especially useful for:
- publishing anchors on-chain
- proving publisher identity

---

## 10) Endpoint security requirements

Recommended security posture per endpoint:

Public (no auth):
- `GET /health`
- `GET /meta`

Authenticated:
- all `/jobs/*`
- all `/bundles/*`
- all `/schemas/*`
- all `/onchain/*`

If the service is strictly private, you may also auth `/meta`.

---

## 11) Configuration example

Example environment variables:

- `SIGNIA_API_KEYS=sk_signia_...;sk_signia_...`
- `SIGNIA_AUTH_MODE=api_key`
- `SIGNIA_RATE_LIMIT_JOBS_PER_MIN=10`
- `SIGNIA_RATE_LIMIT_VERIFY_PER_MIN=60`

Avoid putting secrets in repository files. Use a `.env` file locally only.

---

## 12) Testing

Required tests:
- missing key → 401
- invalid key → 401
- valid key → 200 for protected endpoints
- key is never logged
- rate limits return 429

---

## 13) Related documents

- OpenAPI spec: `docs/api/openapi.yaml`
