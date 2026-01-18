
# Configuration

This document defines configuration for the SIGNIA CLI and the SIGNIA API service. It describes:
- config file formats and precedence
- environment variables
- policy and limit configuration
- plugin configuration
- network and on-chain configuration
- examples for real deployments

Related docs:
- `docs/cli/usage.md`
- `docs/api/auth.md`
- `docs/api/rate-limits.md`
- `docs/onchain/registry-program.md`

---

## 1) Configuration precedence

Highest to lowest precedence:

1. CLI flags
2. Environment variables
3. Project config file (in repo/workdir)
4. User config file (home directory)
5. Built-in defaults

If the same key is set in multiple places, the higher-precedence value wins.

---

## 2) Config file locations

### 2.1 User config
Default path:
- Linux/macOS: `~/.config/signia/config.toml`
- Windows: `%APPDATA%\signia\config.toml`

### 2.2 Project config
SIGNIA looks for a project config in:
- `./signia.toml`
- `./.signia/config.toml`

The nearest config to the working directory is used.

---

## 3) File format

Config is TOML.

Top-level sections:
- `[core]`
- `[policies]`
- `[limits]`
- `[plugins.<id>]`
- `[api]`
- `[onchain]`

---

## 4) Core configuration

### 4.1 [core]
Keys:
- `default_plugin` (string)
- `out_dir` (string)
- `cache_dir` (string)
- `log_level` (string: error|warn|info|debug|trace)
- `json` (bool) default output format preference

Example:

```toml
[core]
default_plugin = "repo"
out_dir = "./out"
cache_dir = "~/.cache/signia"
log_level = "info"
json = false
```

---

## 5) Determinism policies

Policies control normalization and security posture.

### 5.1 [policies]
Keys:
- `policy_version` = "v1"
- `path_root` = "artifact:/"
- `newline` = "lf"
- `encoding` = "utf-8"
- `symlinks` = "deny" | "resolve-within-root"
- `network` = "deny" | "allow-pinned-only"

Example:

```toml
[policies]
policy_version = "v1"
path_root = "artifact:/"
newline = "lf"
encoding = "utf-8"
symlinks = "deny"
network = "deny"
```

Notes:
- In `allow-pinned-only`, remote fetch must include checksum.

---

## 6) Limits

Limits control resource usage.

### 6.1 [limits]
Keys:
- `max_total_bytes` (int)
- `max_file_bytes` (int)
- `max_files` (int)
- `max_depth` (int)
- `max_nodes` (int)
- `max_edges` (int)
- `timeout_ms` (int)

Example:

```toml
[limits]
max_total_bytes = 268435456
max_file_bytes = 10485760
max_files = 20000
max_depth = 64
max_nodes = 200000
max_edges = 400000
timeout_ms = 300000
```

---

## 7) Plugin configuration

Each plugin may expose custom configuration keys. Plugin configuration is hashed into the manifest.

### 7.1 [plugins.<id>]
Example:

```toml
[plugins.repo]
include_globs = ["**/*"]
exclude_globs = ["**/.git/**", "**/target/**"]
emit_digests = true
max_file_bytes_override = 1048576

[plugins.openapi]
strict = true
resolve_refs = "deny-network"
```

Rules:
- plugin config must be deterministic
- do not include timestamps or absolute host paths

---

## 8) API configuration

### 8.1 [api]
Keys:
- `base_url` (string)
- `api_key` (string, secret)
- `timeout_ms` (int)
- `retries` (int)
- `verify_tls` (bool)
- `user_agent` (string)

Example:

```toml
[api]
base_url = "http://localhost:8787"
api_key = "sk_signia_REDACTED"
timeout_ms = 30000
retries = 2
verify_tls = true
user_agent = "signia-cli/0.1.0"
```

Environment variable equivalents:
- `SIGNIA_API_BASE_URL`
- `SIGNIA_API_KEY`
- `SIGNIA_API_TIMEOUT_MS`

Do not commit API keys.

---

## 9) On-chain configuration

### 9.1 [onchain]
Keys:
- `network` = "mainnet-beta" | "devnet" | "testnet" | "localnet"
- `rpc_url` (optional override)
- `program_id` (registry program id)
- `payer_keypair` (path to keypair json)
- `publisher_keypair` (optional)
- `commitment` = "processed" | "confirmed" | "finalized"

Example:

```toml
[onchain]
network = "devnet"
rpc_url = "https://api.devnet.solana.com"
program_id = "SIGNiA111111111111111111111111111111111111"
payer_keypair = "~/.config/solana/id.json"
commitment = "confirmed"
```

Rules:
- do not embed keypair contents; only paths
- keypair paths are local-only and should not appear in generated bundles

---

## 10) CLI flags mapping

Common flag mappings:

- `--plugin` overrides `[core].default_plugin`
- `--out` overrides `[core].out_dir`
- `--safe` sets conservative policy defaults unless overridden
- `--policy-network` overrides `[policies].network`
- `--policy-symlinks` overrides `[policies].symlinks`
- `--limit-max-files` overrides `[limits].max_files`
- `--api-base-url` overrides `[api].base_url`
- `--api-key` overrides `[api].api_key`

---

## 11) Example configurations

### 11.1 Minimal config for local use
```toml
[core]
default_plugin = "repo"

[policies]
policy_version = "v1"
symlinks = "deny"
network = "deny"

[limits]
max_total_bytes = 268435456
timeout_ms = 300000
```

### 11.2 Self-hosted API operator config
```toml
[api]
base_url = "https://signia.internal"
timeout_ms = 60000
retries = 3

[limits]
max_total_bytes = 536870912
max_files = 50000
timeout_ms = 600000
```

### 11.3 CI config (determinism-first)
```toml
[core]
log_level = "info"
json = true

[policies]
policy_version = "v1"
symlinks = "deny"
network = "deny"

[limits]
max_total_bytes = 268435456
max_file_bytes = 5242880
max_files = 20000
timeout_ms = 300000
```

---

## 12) Security guidance

- Do not commit `api_key` values.
- Avoid placing secrets in project config; prefer environment variables.
- Keep policies strict by default (`--safe`).
- Do not enable network unless pinned fetch is required.
- Treat plugin config as part of the trust boundary.

---

## 13) Related documents

- CLI usage: `docs/cli/usage.md`
- API auth: `docs/api/auth.md`
- Rate limits: `docs/api/rate-limits.md`
- Registry: `docs/onchain/registry-program.md`
