
# signia-plugins

The `signia-plugins` crate provides the plugin system and built-in plugins for **SIGNIA**.

Plugins extend SIGNIA by translating external structures into canonical, verifiable
on-chain forms. They are designed to be deterministic, auditable, and sandboxable.

This crate is intended to be used together with `signia-core`.

---

## Goals

- Deterministic execution (same input → same output)
- No hidden side effects
- Explicit inputs and outputs
- Safe plugin isolation
- Clear provenance and diagnostics

---

## Architecture

```
signia-plugins
├── src
│   ├── lib.rs            # Public plugin API
│   ├── plugin.rs         # Plugin trait definitions
│   ├── registry.rs       # Plugin registry and resolution
│   ├── builtin/          # Built-in plugins
│   │   ├── mod.rs
│   │   ├── repo.rs       # Git repository plugin
│   │   ├── dataset.rs    # Dataset plugin
│   │   └── openapi.rs    # OpenAPI plugin
│   └── sandbox/          # Optional WASM sandbox
│       └── mod.rs
```

---

## Plugin Model

A plugin is a pure transformation:

```
Input Structure
      ↓
[ Plugin ]
      ↓
Canonical IR → Schema / Manifest / Proof
```

Plugins **must not**:
- Perform network access unless explicitly allowed
- Read system time or environment variables
- Produce nondeterministic output

---

## Plugin Trait (Simplified)

```rust
use signia_core::pipeline::context::PipelineContext;

pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn supports(&self, input_type: &str) -> bool;

    fn execute(&self, ctx: &mut PipelineContext) -> Result<(), anyhow::Error>;
}
```

---

## Built-in Plugins

Built-in plugins are enabled by default.

| Plugin | Description |
|------|------------|
| `repo` | Converts Git repositories into structured graphs |
| `dataset` | Converts datasets into canonical schemas |
| `openapi` | Converts OpenAPI specs into on-chain schemas |

---

## WASM Sandboxing (Optional)

Enable the `wasm` feature to run plugins in a WASM sandbox:

```toml
signia-plugins = { version = "0.1.0", features = ["wasm"] }
```

Sandboxed plugins:
- Have no filesystem access
- Have no network access
- Are limited by fuel and memory quotas

---

## Determinism Guarantees

Plugins are expected to:
- Use canonical JSON serialization
- Use stable sorting
- Avoid floating-point arithmetic
- Declare all limits explicitly

Violations are reported via diagnostics.

---

## Development

### Build

```bash
cargo build -p signia-plugins
```

### Test

```bash
cargo test -p signia-plugins
```

---

## Security

This crate is part of the SIGNIA trusted computing base.

If you discover a vulnerability, please follow the responsible disclosure
process described in the main SIGNIA repository.

---

## License

Licensed under the Apache License, Version 2.0.
