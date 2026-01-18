
# Installation

This document explains how to install the SIGNIA CLI and optional components. It covers:
- installing from source
- installing from prebuilt binaries
- toolchain requirements
- Docker-based usage
- verifying the installation
- troubleshooting

Related docs:
- `docs/cli/quickstart.md` (if present)
- `docs/cli/commands.md` (if present)
- `docs/determinism/reproducible-builds.md`

---

## 1) Supported environments

SIGNIA CLI is designed to run on:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64) via WSL2 (recommended) or native (best-effort)

For deterministic builds and easiest setup, Linux is recommended.

---

## 2) Install from prebuilt binaries

If the repository publishes GitHub Releases with artifacts, you can install from the release page.

### 2.1 Download
Download the artifact matching your OS/arch:
- `signia-<version>-x86_64-unknown-linux-gnu.tar.gz`
- `signia-<version>-aarch64-apple-darwin.tar.gz`
- etc.

Unpack and place `signia` in your PATH.

Linux/macOS:

```bash
tar -xzf signia-<version>-<target>.tar.gz
sudo mv signia /usr/local/bin/signia
signia --version
```

Windows (WSL2):
- unpack in WSL2 and place into `/usr/local/bin`.

### 2.2 Verify checksums (recommended)
If releases include `SHA256SUMS` and signature:
- verify checksum
- verify signature (optional)

Example:

```bash
sha256sum -c SHA256SUMS
```

---

## 3) Install from source

### 3.1 Requirements
- Rust toolchain (stable)
- Git
- Optional: Node.js (for web UI / client packages)
- Optional: Docker (for sandboxed runs)

### 3.2 Rust toolchain
Install Rust via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add clippy rustfmt
```

### 3.3 Clone and build
```bash
git clone https://github.com/<your-org>/signia.git
cd signia
cargo build --release
./target/release/signia --version
```

### 3.4 Install to PATH
```bash
cargo install --path crates/signia-cli --locked
signia --help
```

If your repo uses a workspace layout, the install path may differ. The intention is:
- build the CLI crate
- install the resulting binary

---

## 4) Docker-based installation (recommended for consistent sandbox)

If you want consistent execution and isolation, use Docker.

### 4.1 Build the image
```bash
docker build -t signia:local .
```

### 4.2 Run
```bash
docker run --rm -it signia:local signia --version
```

### 4.3 Mount inputs/outputs
```bash
mkdir -p ./out
docker run --rm -it \
  -v "$(pwd)/examples:/work/in:ro" \
  -v "$(pwd)/out:/work/out:rw" \
  --network=none \
  signia:local \
  signia compile --input /work/in --out /work/out --safe
```

Notes:
- `--network=none` enforces the default sandbox posture.
- Use read-only mount for inputs when possible.

---

## 5) Optional components

### 5.1 SIGNIA API service
If you run the API:
- see `docs/api/openapi.yaml`
- see `docs/api/auth.md`

Typical local run (example):
```bash
cargo run -p signia-api -- --bind 0.0.0.0:8787
```

### 5.2 On-chain registry program
If you deploy the Solana program:
- see `docs/onchain/registry-program.md`
- see `docs/onchain/instructions.md`

---

## 6) Verifying your installation

### 6.1 Basic check
```bash
signia --version
signia --help
```

### 6.2 Run a fixture compile (if included)
If the repo includes fixtures:

```bash
signia compile \
  --plugin openapi \
  --input fixtures/openapi/petstore \
  --out /tmp/signia-out \
  --safe

signia verify --bundle /tmp/signia-out
```

You should see:
- schema hash printed
- verification ok

### 6.3 Determinism check
Run compile twice and compare outputs:

```bash
rm -rf /tmp/signia-out1 /tmp/signia-out2
signia compile --plugin openapi --input fixtures/openapi/petstore --out /tmp/signia-out1 --safe
signia compile --plugin openapi --input fixtures/openapi/petstore --out /tmp/signia-out2 --safe
diff -r /tmp/signia-out1 /tmp/signia-out2
```

---

## 7) Troubleshooting

### 7.1 Cargo build failures
Common fixes:
- update toolchain: `rustup update`
- clear build cache: `cargo clean`
- ensure dependencies are available (OpenSSL on some Linux distros)

### 7.2 Permission issues in Docker
If output directory is owned by root, run container with your user id:

```bash
docker run --rm -it \
  -u "$(id -u):$(id -g)" \
  -v "$(pwd)/out:/work/out:rw" \
  signia:local \
  signia --version
```

### 7.3 File path and newline issues
SIGNIA canonicalizes newlines and paths. If you see drift across OS:
- ensure inputs are identical (line endings, encoding)
- prefer running inside Docker for stability

### 7.4 Network fetch errors
If you enabled network for pinned fetch:
- confirm checksum is correct
- ensure your allowlist includes the host (if configured)
- ensure timeouts are configured

---

## 8) Uninstall

If installed via cargo:
```bash
cargo uninstall signia
```

If installed by moving binary:
- remove `signia` from PATH location.

---

## 9) Version compatibility

- CLI and schema formats are versioned.
- `schema.json` and `manifest.json` include `version` fields.
- Use `signia --version` and `signia doctor` (if available) to verify compatibility.

---

## 10) Related documents

- Reproducible builds: `docs/determinism/reproducible-builds.md`
- API auth: `docs/api/auth.md`
- Registry program: `docs/onchain/registry-program.md`
