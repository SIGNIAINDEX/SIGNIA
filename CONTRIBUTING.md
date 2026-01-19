# Contributing to SIGNIA

## Quick Start

1. Fork and create a branch:
   - `git checkout -b feat/your-change`
2. Install toolchains:
   - `./scripts/bootstrap.sh`
3. Run checks:
   - `make lint`
   - `make test`
4. Open a Pull Request.

## Project Layout

- `crates/` Rust workspace crates
- `programs/` Solana programs (Anchor)
- `sdk/` TypeScript + Python SDKs
- `console/` Web console + interface service
- `schemas/` Canonical JSON schemas

## Style

- Rust: `cargo fmt`, `cargo clippy`
- TypeScript: Biome + TypeScript strict
