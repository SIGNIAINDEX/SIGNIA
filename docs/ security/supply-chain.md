
# Supply Chain Security

This document describes supply chain security practices for the SIGNIA repository. It covers threats, controls, CI/CD hardening, dependency policies, release integrity, and operational recommendations.

SIGNIA is a hash-addressed, determinism-first system. Supply chain security is part of the core trust model because:
- compromised dependencies can subvert determinism and integrity
- CI pipelines produce artifacts consumed by others
- release binaries and containers must be verifiable

---

## 1) Goals

1. Prevent introduction of malicious or compromised dependencies.
2. Reduce blast radius of build tooling and CI execution.
3. Ensure release artifacts are reproducible and verifiable.
4. Provide clear policies for reviewing dependency changes.
5. Detect suspicious changes early via automation.

---

## 2) Threats

### 2.1 Dependency compromise
- Malicious updates published to registries (crates.io, npm)
- Hijacked maintainer accounts
- Typosquatting packages
- Dependency confusion between public/private registries

### 2.2 Build and CI compromise
- Malicious GitHub Actions or compromised runner
- Unsafe workflow permissions
- Secrets exfiltration via untrusted PR workflows

### 2.3 Artifact tampering
- Replacing release binaries or containers post-build
- Man-in-the-middle during artifact download (users not verifying checksums)
- Cache poisoning in build systems

### 2.4 Source integrity degradation
- Force-push rewriting release history
- Unsigned tags
- Unreviewed changes to canonicalization or hashing rules

---

## 3) Baseline controls (required)

### 3.1 Lockfiles and deterministic builds
- Commit and maintain:
  - `Cargo.lock`
  - `pnpm-lock.yaml` (or equivalent)
- Use `--locked` for Rust builds in CI.
- Use `--frozen-lockfile` for pnpm installs in CI.
- Avoid optional network fetches during compilation runs unless explicitly configured.

### 3.2 Branch protection and required reviews
Recommended GitHub settings:
- Protect `main`
- Require PR reviews
- Require status checks (CI, CodeQL, lint/test)
- Require signed commits (optional but recommended)
- Restrict who can push to protected branches

### 3.3 Least privilege workflow permissions
- Default `permissions: read-all` if possible.
- Grant only what is required per workflow job.
- Avoid write permissions in PR workflows.
- Avoid exposing secrets to forks.

### 3.4 Trusted actions policy
- Prefer first-party actions (`actions/*`, `github/*`) and widely adopted vendors.
- Pin action versions to major versions at minimum (`@v4`, `@v3`).
- For higher assurance, pin by commit SHA for critical release workflows.

---

## 4) Rust dependency policy

### 4.1 Allowed sources
- Prefer crates.io.
- Avoid Git dependencies unless justified.
- If Git dependencies are required:
  - pin to a specific commit SHA
  - document why in the PR description

### 4.2 Auditing
- Use `cargo-audit` to detect known vulnerabilities.
- Use `cargo-deny` to enforce:
  - license allowlist
  - banned crates
  - duplicate dependencies policy
  - source policy (no unpinned git)

### 4.3 Review rules for dependency changes
Any PR that changes `Cargo.lock` must:
- describe why dependencies changed
- list new crates and their purpose
- confirm `cargo audit` passes (or document exceptions)
- confirm licensing is compatible with repository policy

---

## 5) Node dependency policy (pnpm)

### 5.1 Allowed sources
- Prefer npm registry packages.
- Avoid installing from Git URLs or tarball URLs.
- Avoid `postinstall` scripts unless required.
- Prefer `pnpm` with lockfile enforcement.

### 5.2 Auditing
- Run:
  - `pnpm audit` (or equivalent) in CI
  - dependency review via GitHub (Dependency Review Action)

### 5.3 Review rules for JS changes
Any PR that changes `pnpm-lock.yaml` must:
- describe why dependencies changed
- list new packages and their purpose
- confirm no new risky lifecycle scripts are introduced
- confirm the build remains deterministic

---

## 6) GitHub Actions hardening

### 6.1 PR workflow safety
Recommendations:
- Do not use secrets in workflows triggered by forks.
- Use `pull_request` vs `pull_request_target` unless you fully understand risks.
- If `pull_request_target` is required, never checkout untrusted code before protecting secrets.

### 6.2 Dependency Review
Enable dependency review for PRs:
- fail PRs introducing known vulnerable dependencies
- require explicit approvals for new packages

### 6.3 Concurrency and cancellation
Use concurrency controls to reduce resource waste and limit attack surface from repeated runs.

### 6.4 Artifact retention
- Upload logs and test artifacts for debugging.
- Avoid uploading sensitive artifacts.
- Set reasonable retention.

---

## 7) Release integrity

### 7.1 Signed tags and provenance (recommended)
- Use signed tags for releases.
- Consider Sigstore or similar provenance tooling if adopting strong verification.
- Document the release process in `RELEASING.md` (recommended).

### 7.2 Checksums
- Release binaries must include SHA256 checksums.
- Users should verify checksums before use.

### 7.3 Container integrity
- Publish containers to a trusted registry (e.g., GHCR).
- Prefer SBOM and provenance generation where possible.
- Tag containers with:
  - version tag (release)
  - commit SHA tag
  - `latest` (optional, only from main)

### 7.4 Reproducibility targets
Ideal target:
- independent builders can reproduce identical binaries (or at least identical hashes for canonical outputs)
Practical target:
- deterministic outputs and verification correctness are reproducible, even if binary reproduction varies by toolchain.

---

## 8) Canonicalization and hashing change control

Because SIGNIA depends on stable hashes:
- changes to hash domains, canonical JSON encoding, or ordering rules are security-sensitive
- they must be:
  - explicitly versioned
  - documented in `docs/`
  - covered by golden fixtures
  - reviewed by maintainers

Recommended policy:
- Treat these changes as requiring “security review” label and additional approvals.

---

## 9) Recommended automation (CI checks)

### 9.1 Required checks
- Rust:
  - `cargo fmt --check`
  - `cargo clippy -D warnings`
  - `cargo test --all-features --locked`
  - `cargo audit`
  - `cargo deny check`
- Node:
  - `pnpm lint`
  - `pnpm typecheck`
  - `pnpm test`
  - dependency review
- CodeQL:
  - Rust and JS/TS analysis
- E2E:
  - compile → verify → (optional publish) → (optional fetch)

### 9.2 Optional checks
- Fuzzing (parsers/canonicalization)
- SAST policies beyond CodeQL
- Secret scanning (GitHub Advanced Security if available)

---

## 10) Secrets management

Rules:
- Never store secrets in the repository.
- Use GitHub Secrets for CI, scoped to environments where needed.
- Avoid printing environment variables.
- Ensure logs do not contain tokens.

Recommended secrets (only if needed):
- `NPM_TOKEN` for npm publishing
- Container registry credentials (if not using GHCR with `GITHUB_TOKEN`)
- Solana deploy keys (only for controlled release workflows)

For Solana:
- Do not store private keys in plain text.
- Use secure secret storage and consider hardware-backed keys.

---

## 11) Local developer safety

Recommendations:
- Use `cargo install --locked` where possible.
- Avoid running untrusted plugin inputs without sandbox limits.
- Prefer containerized builds for reproducibility.
- Keep tooling updated and pinned in `.tool-versions` or equivalent if used.

---

## 12) Incident response

If a dependency compromise is suspected:
1. Freeze releases.
2. Identify the dependency and affected versions.
3. Revoke credentials if applicable.
4. Publish a security advisory and pinned mitigations.
5. Update lockfiles and audit results.
6. Add new CI checks if the compromise path was not detected.

For malicious PR attempts:
- rotate any exposed secrets immediately
- review workflow permissions and triggers
- consider restricting CI for forked PRs

---

## 13) Practical checklist

### For maintainers
- [ ] Branch protection enabled for `main`
- [ ] Required status checks configured
- [ ] Dependency review enabled
- [ ] CodeQL enabled
- [ ] Release workflow uses checksums
- [ ] Actions permissions are least-privilege
- [ ] Security policy is published (`SECURITY.md`)

### For contributors
- [ ] Explain dependency changes in PRs
- [ ] Do not introduce unpinned git deps
- [ ] Avoid adding new build scripts unless necessary
- [ ] Add tests for determinism-related changes
- [ ] Keep lockfiles consistent with changes

---

## 14) Disclaimer

There is currently **no token issued**.

Do not treat any token claims as part of this repository unless explicitly documented in official releases and channels.
