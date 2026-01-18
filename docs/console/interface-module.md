
# Console Interface Module

This document specifies the Console interface module: the boundary between the SIGNIA Console UI and the SIGNIA API. It defines:
- typed request/response contracts used by the UI
- an adapter layer for transport (fetch, retries, auth)
- error normalization (mapping HTTP errors to stable codes)
- caching and pagination helpers
- offline/local verification hooks (optional)

This is intended as a practical blueprint for a real, runnable Console implementation.

Related docs:
- `docs/api/openapi.yaml`
- `docs/api/error-codes.md`
- `docs/api/auth.md`
- `docs/api/rate-limits.md`

---

## 1) Goals

1. The Console should not spread raw `fetch()` calls across the UI.
2. All API calls should go through a single module with consistent:
   - auth injection
   - timeout and retry policy
   - rate limit handling
   - error normalization
3. The module should be testable with mocked transports.
4. The module should enable gradual addition of features (on-chain, plugins, etc.).

---

## 2) Module shape

Recommended file:
- `apps/console/src/lib/signiaClient.ts`

Exports:
- `createSigniaClient(config): SigniaClient`
- `SigniaClient` interface
- typed DTOs (or import generated types)
- `SigniaError` type

The UI should import only the interface and a factory:
- `const client = createSigniaClient({ baseUrl, apiKey })`

---

## 3) Client configuration

### 3.1 Config fields

```ts
export type SigniaClientConfig = {
  baseUrl: string;              // e.g. http://localhost:8787
  apiKey?: string;              // X-API-Key, optional for public endpoints
  timeoutMs?: number;           // default 30000
  retries?: number;             // default 2
  retryBackoffMs?: number;      // default 250
  userAgent?: string;           // optional, for server logs
};
```

Rules:
- baseUrl must not end with `/`
- store apiKey only in memory by default (persisting is optional)

---

## 4) Error normalization

All errors returned to the UI should be normalized:

```ts
export type SigniaError = {
  code: string;                 // stable error code
  message: string;              // human-readable
  status?: number;              // HTTP status
  requestId?: string;           // server request id if provided
  retryAfterSeconds?: number;   // if rate limited
  details?: Record<string, unknown>;
};
```

Normalization logic:
1. If server returns JSON with `{ error: { code, message, details } }`, use it.
2. Otherwise map by HTTP status:
   - 400 -> INPUT_BAD_REQUEST
   - 401 -> AUTH_INVALID_API_KEY (or AUTH_MISSING_API_KEY if no key)
   - 403 -> AUTH_FORBIDDEN
   - 404 -> NOT_FOUND
   - 409 -> CONFLICT
   - 413 -> INPUT_TOO_LARGE
   - 429 -> RATE_LIMITED
   - 500 -> INTERNAL_ERROR

Include `Retry-After` if present.

---

## 5) Transport adapter

The client should implement a request helper:

Features:
- inject headers: `X-API-Key`, `User-Agent`, `Content-Type`
- enforce timeout via `AbortController`
- retries on:
  - network errors
  - 429 (with Retry-After)
  - 503 (optional)
- no retries on:
  - 4xx except 429

Pseudo behavior:
- attempt request
- if ok -> return JSON or blob
- else parse error and throw `SigniaError`

---

## 6) Typed API surface for the Console

### 6.1 Jobs
The module should expose:

- `createJob(req): Promise<{ jobId; status }>`
- `uploadJobInput(jobId, file, kind?): Promise<{ inputId; sha256 }>`
- `runJob(jobId): Promise<{ jobId; status }>`
- `getJob(jobId): Promise<Job>`
- `listJobs(params): Promise<{ items; nextCursor? }>`

### 6.2 Bundles
- `uploadBundle(file): Promise<{ bundleId; schemaHash; proofRoot }>`
- `getBundle(bundleId): Promise<Bundle>`
- `downloadBundle(bundleId): Promise<Blob>`
- `verifyBundle(bundleId, req?): Promise<VerifyResult>`
- `listBundles(params): Promise<{ items; nextCursor? }>`

### 6.3 Schemas
- `listSchemas(params): Promise<{ items; nextCursor? }>`
- `getSchema(schemaHash): Promise<Schema>`
- `getSchemaJson(schemaHash): Promise<object>`
- `getSchemaManifest(schemaHash): Promise<object>`
- `getSchemaProof(schemaHash): Promise<object>`

### 6.4 On-chain (optional)
- `getOnchainRecord(schemaHash, network?): Promise<OnchainRecord>`
- `publishOnchain(req): Promise<PublishResponse>`

---

## 7) Pagination helper

The client should provide a generic pagination helper:

```ts
export type Page<T> = { items: T[]; nextCursor?: string | null };

export async function* paginate<T>(
  fetchPage: (cursor?: string | null) => Promise<Page<T>>,
): AsyncGenerator<T, void, unknown> {
  let cursor: string | null | undefined = null;
  while (true) {
    const page = await fetchPage(cursor);
    for (const item of page.items) yield item;
    if (!page.nextCursor) break;
    cursor = page.nextCursor;
  }
}
```

This makes large lists easy to consume without overfetching.

---

## 8) Caching strategy

Recommended caching for UI responsiveness:
- memory cache keyed by:
  - schemaHash -> schema metadata
  - bundleId -> bundle metadata
- short TTL (e.g., 30s to 120s)
- invalidate on:
  - successful upload
  - publish operations

Do not cache:
- errors
- auth failures

---

## 9) Offline verification hook (optional)

If you implement browser-side verification:
- provide a `verifyLocalBundle(bundleBytes)` method
- use a WASM module or pure JS implementation for hashing and canonical JSON

The interface module can expose:

```ts
export type LocalVerifier = {
  verifyBundleArchive(bytes: Uint8Array): Promise<VerifyResult>;
};

export function attachLocalVerifier(client: SigniaClient, verifier: LocalVerifier): SigniaClient;
```

This keeps UI code consistent.

---

## 10) Testing strategy

### 10.1 Unit tests
Mock the transport layer:
- provide a fake fetch implementation
- assert:
  - headers are added
  - timeout aborts
  - errors normalize correctly
  - retries happen as expected

### 10.2 Contract tests
Run against a local API container:
- verify the module matches OpenAPI
- validate JSON shapes
- capture rate limit headers behavior

---

## 11) Example TypeScript skeleton (runnable)

Below is a compact example skeleton. A real repo can generate types from OpenAPI, but this is enough for a functional module.

```ts
// apps/console/src/lib/signiaClient.ts
export type SigniaClientConfig = {
  baseUrl: string;
  apiKey?: string;
  timeoutMs?: number;
  retries?: number;
  retryBackoffMs?: number;
  userAgent?: string;
};

export type SigniaError = {
  code: string;
  message: string;
  status?: number;
  requestId?: string;
  retryAfterSeconds?: number;
  details?: Record<string, unknown>;
};

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function parseJsonSafe(resp: Response): Promise<any | null> {
  const text = await resp.text();
  if (!text) return null;
  try { return JSON.parse(text); } catch { return { _raw: text }; }
}

function normalizeError(status: number, payload: any, headers: Headers, hasApiKey: boolean): SigniaError {
  const retryAfter = headers.get("retry-after");
  const requestId = headers.get("x-request-id") ?? undefined;

  if (payload?.error?.code && payload?.error?.message) {
    return {
      code: String(payload.error.code),
      message: String(payload.error.message),
      status,
      requestId,
      retryAfterSeconds: retryAfter ? Number(retryAfter) : undefined,
      details: payload.error.details ?? undefined,
    };
  }

  const mapped =
    status === 400 ? "INPUT_BAD_REQUEST" :
    status === 401 ? (hasApiKey ? "AUTH_INVALID_API_KEY" : "AUTH_MISSING_API_KEY") :
    status === 403 ? "AUTH_FORBIDDEN" :
    status === 404 ? "NOT_FOUND" :
    status === 409 ? "CONFLICT" :
    status === 413 ? "INPUT_TOO_LARGE" :
    status === 429 ? "RATE_LIMITED" :
    status >= 500 ? "INTERNAL_ERROR" :
    "UNKNOWN_ERROR";

  return {
    code: mapped,
    message: payload?.error?.message ?? `Request failed with status ${status}`,
    status,
    requestId,
    retryAfterSeconds: retryAfter ? Number(retryAfter) : undefined,
    details: payload?.error?.details ?? undefined,
  };
}

async function request<T>(
  cfg: Required<Pick<SigniaClientConfig, "baseUrl" | "timeoutMs" | "retries" | "retryBackoffMs">> & SigniaClientConfig,
  init: RequestInit & { path: string; expect?: "json" | "blob" },
): Promise<T> {
  const url = cfg.baseUrl + init.path;
  const expect = init.expect ?? "json";
  const retries = cfg.retries;

  for (let attempt = 0; attempt <= retries; attempt++) {
    const controller = new AbortController();
    const t = setTimeout(() => controller.abort(), cfg.timeoutMs);

    try {
      const headers = new Headers(init.headers ?? {});
      if (cfg.apiKey) headers.set("X-API-Key", cfg.apiKey);
      if (cfg.userAgent) headers.set("User-Agent", cfg.userAgent);

      const resp = await fetch(url, { ...init, headers, signal: controller.signal });
      clearTimeout(t);

      if (resp.ok) {
        if (expect === "blob") return (await resp.blob()) as any;
        return (await resp.json()) as T;
      }

      const payload = await parseJsonSafe(resp);
      const err = normalizeError(resp.status, payload, resp.headers, Boolean(cfg.apiKey));

      // Retry policy
      const retryable = resp.status === 429 || resp.status === 503;
      if (attempt < retries && retryable) {
        const wait = err.retryAfterSeconds ? err.retryAfterSeconds * 1000 : cfg.retryBackoffMs * (attempt + 1);
        await sleep(wait);
        continue;
      }

      throw err;
    } catch (e: any) {
      clearTimeout(t);
      const isAbort = e?.name === "AbortError";
      const isNetwork = e instanceof TypeError; // fetch network error pattern

      if (attempt < retries && (isAbort || isNetwork)) {
        await sleep(cfg.retryBackoffMs * (attempt + 1));
        continue;
      }

      if (e?.code) throw e; // already SigniaError
      throw { code: "INTERNAL_ERROR", message: e?.message ?? "Unknown error" } as SigniaError;
    }
  }

  throw { code: "INTERNAL_ERROR", message: "Unreachable" } as SigniaError;
}

export type SigniaClient = {
  createJob: (body: any) => Promise<any>;
  uploadJobInput: (jobId: string, file: File, kind?: "archive" | "file") => Promise<any>;
  runJob: (jobId: string) => Promise<any>;
  getJob: (jobId: string) => Promise<any>;
  verifyBundle: (bundleId: string, body?: any) => Promise<any>;
  downloadBundle: (bundleId: string) => Promise<Blob>;
};

export function createSigniaClient(config: SigniaClientConfig): SigniaClient {
  const cfg = {
    ...config,
    timeoutMs: config.timeoutMs ?? 30000,
    retries: config.retries ?? 2,
    retryBackoffMs: config.retryBackoffMs ?? 250,
  };

  return {
    createJob: (body) => request<any>(cfg as any, {
      path: "/jobs",
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      expect: "json",
    }),

    uploadJobInput: (jobId, file, kind = "archive") => {
      const form = new FormData();
      form.append("kind", kind);
      form.append("file", file, file.name);
      return request<any>(cfg as any, {
        path: `/jobs/${encodeURIComponent(jobId)}/inputs`,
        method: "POST",
        body: form,
        expect: "json",
      });
    },

    runJob: (jobId) => request<any>(cfg as any, {
      path: `/jobs/${encodeURIComponent(jobId)}/run`,
      method: "POST",
      expect: "json",
    }),

    getJob: (jobId) => request<any>(cfg as any, {
      path: `/jobs/${encodeURIComponent(jobId)}`,
      method: "GET",
      expect: "json",
    }),

    verifyBundle: (bundleId, body) => request<any>(cfg as any, {
      path: `/bundles/${encodeURIComponent(bundleId)}/verify`,
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: body ? JSON.stringify(body) : undefined,
      expect: "json",
    }),

    downloadBundle: (bundleId) => request<Blob>(cfg as any, {
      path: `/bundles/${encodeURIComponent(bundleId)}/download`,
      method: "GET",
      expect: "blob",
    }),
  };
}
```

This module is small but production-shaped:
- timeouts
- retries
- error normalization
- type surface

A full implementation would generate types from OpenAPI and add all endpoints.

---

## 12) Related documents

- Error codes: `docs/api/error-codes.md`
- Auth: `docs/api/auth.md`
- Rate limits: `docs/api/rate-limits.md`
- OpenAPI: `docs/api/openapi.yaml`
