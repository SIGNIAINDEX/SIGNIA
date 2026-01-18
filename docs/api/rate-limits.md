
# Rate Limits

This document defines rate limiting policy for the SIGNIA API service. Rate limits protect the service from abuse and ensure predictable performance for all tenants.

This doc covers:
- what is rate-limited
- default quotas and burst behavior
- limit keys (API key, IP, tenant)
- response headers and error shape
- operational tuning and monitoring

Related docs:
- `docs/api/auth.md`
- `docs/api/openapi.yaml`

---

## 1) Goals

1. Prevent abuse and resource exhaustion.
2. Provide predictable service quality.
3. Make limits transparent to clients via standard headers.
4. Support multi-tenant deployments.

---

## 2) Rate limiting model

### 2.1 Limit dimensions
Rate limiting should consider:
- API key (primary identity)
- IP address (secondary, for abuse mitigation)
- tenant id (optional)
- endpoint category (jobs, uploads, downloads, verify, onchain)

Default limit key:
- `api_key` (or `ip` if no key is present for public endpoints)

### 2.2 Algorithms
Recommended algorithms:
- Token bucket for request rate
- Leaky bucket for smoothing
- Separate concurrency limits for expensive operations

### 2.3 Burst behavior
Burst is allowed up to a configured capacity. After burst is consumed, requests are throttled to steady rate.

Example:
- steady: 10 requests/min
- burst: 20
A client can do 20 quickly, then must wait for tokens to refill.

---

## 3) Endpoint categories and defaults

These defaults are conservative and intended for a single-node deployment. Scale them up as you add capacity.

### 3.1 System endpoints
Public:
- `GET /health`
- `GET /meta`

Defaults:
- 120 req/min per IP
- burst 240

### 3.2 Job management
Endpoints:
- `POST /jobs`
- `GET /jobs`
- `GET /jobs/{jobId}`
- `DELETE /jobs/{jobId}`
- `POST /jobs/{jobId}/run`

Defaults (per API key):
- `POST /jobs`: 10/min, burst 20
- `GET /jobs*`: 120/min, burst 240
- `DELETE /jobs/{jobId}`: 30/min, burst 60
- `POST /jobs/{jobId}/run`: 20/min, burst 40

### 3.3 Input uploads
Endpoint:
- `POST /jobs/{jobId}/inputs`
- `POST /bundles` (bundle upload)

Uploads should be limited by:
- request rate
- bytes per time
- max size per request

Defaults (per API key):
- upload requests: 30/min, burst 60
- upload bytes: 1 GiB/day
- max request size:
  - input upload: 256 MiB
  - bundle upload: 64 MiB

### 3.4 Bundle reads
Endpoints:
- `GET /bundles`
- `GET /bundles/{bundleId}`
- `GET /bundles/{bundleId}/download`

Defaults (per API key):
- metadata reads: 240/min, burst 480
- downloads: 60/min, burst 120
- download bytes: 10 GiB/day (optional quota)

### 3.5 Verification
Endpoints:
- `POST /bundles/{bundleId}/verify`
- (optional future) `POST /verify` (direct upload verify)

Verification is CPU-heavy and should be limited.

Defaults (per API key):
- 60/min, burst 120
- concurrency limit: 4 in-flight verifies per key

### 3.6 Schema queries
Endpoints:
- `GET /schemas`
- `GET /schemas/{schemaHash}/*`

Defaults (per API key):
- 240/min, burst 480

### 3.7 On-chain operations
Endpoints:
- `GET /onchain/records/{schemaHash}`
- `POST /onchain/publish`

Defaults (per API key):
- lookups: 120/min, burst 240
- publish: 10/min, burst 20
- concurrency limit for publish: 2 in-flight per key

---

## 4) Concurrency limits (recommended)

Request rate limits do not protect long-running tasks.

Add concurrency limits for:
- compilation runs
- verification
- on-chain publish

Suggested defaults:
- compile jobs: 2 running jobs per key
- verification: 4 in-flight per key
- publish: 2 in-flight per key

If exceeded:
- return `429` with a clear error code (see Section 6).

---

## 5) Quotas (optional)

Quotas help control cost:
- bytes uploaded per day
- bytes downloaded per day
- total job runtime per day
- total verification CPU seconds per day

Quotas can be implemented with:
- rolling windows (24h)
- reset at midnight UTC (simpler)

Defaults (single-node):
- upload bytes: 1 GiB/day per key
- download bytes: 10 GiB/day per key

When exceeded:
- return `429` or `402` depending on your billing model
- this spec uses `429` for simplicity

---

## 6) Response headers and error shape

### 6.1 Headers
For rate-limited endpoints, include:
- `X-RateLimit-Limit`: steady rate limit for the bucket
- `X-RateLimit-Remaining`: remaining tokens (approximate)
- `X-RateLimit-Reset`: unix timestamp when next token is expected or window resets
- `Retry-After`: seconds until retry is reasonable

If you implement multiple buckets (rate + bytes + concurrency), you may include:
- `X-RateLimit-Policy`: human-readable policy id
- `X-RateLimit-Reason`: which bucket triggered (rate, bytes, concurrency)

### 6.2 Error response
Return:
- HTTP 429
- JSON body:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded",
    "details": {
      "policy": "jobs:create",
      "retryAfterSeconds": 12
    }
  }
}
```

Error codes recommended:
- `RATE_LIMITED`
- `QUOTA_EXCEEDED`
- `CONCURRENCY_LIMITED`

---

## 7) Bypass and internal keys

You may create internal keys with elevated limits for:
- operators
- CI pipelines
- trusted automation

Rules:
- elevated keys must still have limits
- log usage and protect keys like secrets

---

## 8) Monitoring and alerting

Track:
- requests by endpoint and status code
- 429 rate by key and IP
- average latency per endpoint
- concurrency utilization
- bytes uploaded/downloaded
- job queue depth

Alerts:
- sustained high 429s for many keys (may indicate under-provisioning)
- a single key generating many 429s (abuse or bug)
- upload bytes spikes

---

## 9) Tuning guidance

If users report frequent throttling:
- increase burst capacity first
- then increase steady rates
- then add more workers/nodes

For compilation:
- prefer limiting concurrent jobs rather than raw request rate
- implement a queue with fair scheduling per key

For verification:
- use caching keyed by schema hash and proof root
- only recompute hashes when needed

---

## 10) Implementation notes (practical)

### 10.1 Buckets per endpoint category
Define categories:
- `system`
- `jobs_create`
- `jobs_read`
- `upload`
- `bundle_read`
- `download`
- `verify`
- `schemas_read`
- `onchain_lookup`
- `onchain_publish`

Map routes to categories.

### 10.2 Storage for counters
Options:
- in-memory (single node)
- Redis (multi node)
- Postgres (slower, but durable)

For distributed deployments, Redis is typical.

### 10.3 Clock source
Use a monotonic clock for refill calculations. Avoid wall-clock drift issues.

---

## 11) Example default policy table

This is a reference summary:

- system: 120/min (IP), burst 240
- jobs:create: 10/min, burst 20
- jobs:read: 120/min, burst 240
- upload: 30/min, burst 60 + size limits
- bundle:read: 240/min, burst 480
- download: 60/min, burst 120
- verify: 60/min, burst 120 + 4 concurrency
- schemas:read: 240/min, burst 480
- onchain:lookup: 120/min, burst 240
- onchain:publish: 10/min, burst 20 + 2 concurrency

---

## 12) Related documents

- Authentication: `docs/api/auth.md`
- OpenAPI: `docs/api/openapi.yaml`
