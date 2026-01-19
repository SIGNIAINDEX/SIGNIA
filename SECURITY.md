# Security Policy

This document explains how to report security vulnerabilities in SIGNIA and how we handle disclosures.

> **Important:** Please do not report security vulnerabilities through public GitHub issues.

---

## Supported versions

Security fixes are provided for:

- The `main` branch (active development)
- The most recent tagged release line (if releases exist)

If you are using an older tag, you may be asked to reproduce on a more recent version.

---

## Reporting a vulnerability

### Preferred: GitHub Security Advisories

Use the repository's **Security** tab → **Report a vulnerability** (Private Vulnerability Reporting / Security Advisories).

This is the fastest path and keeps details private.

### Alternative: email

Email reports can be sent to:

- `security@signia.example` (placeholder — replace before publishing if you will use email)

If you report by email, please encrypt sensitive details when possible (PGP) and include an alternate contact method if needed.

---

## What to include

Please include as much of the following as possible:

- Affected component(s): `signia-core`, `signia-plugins`, `signia-store`, `signia-api`, `signia-cli`, `signia-solana-client`, `signia-registry`, SDKs, console, infra
- Affected version(s) or commit SHA(s)
- Reproduction steps (minimum viable PoC)
- Expected vs actual behavior
- Impact assessment (confidentiality / integrity / availability)
- Attack prerequisites (auth required? local only? remote reachable?)
- Logs, traces, sample inputs, or artifacts (redact secrets)
- Mitigations or suggested fix (optional)

If your finding affects Solana program safety, include:

- instruction name(s)
- PDA derivations involved
- account constraints / signer requirements
- any potential privilege escalation

---

## Vulnerability categories (examples)

- Remote code execution in plugin sandbox or interface service
- SSRF, path traversal, or deserialization issues in API/CLI
- Authentication bypass or authorization flaws
- Supply-chain compromise (dependencies, build scripts, CI workflows)
- Determinism breaks causing inconsistent hashes/proofs
- Cryptographic misuse (hashing, Merkle proofs, signing)
- Anchor/Solana program bugs: missing constraints, PDA collisions, authority bypass

---

## Confidentiality and scope

We treat vulnerability reports as confidential. Please avoid disclosing details publicly until:

- a fix is merged, and
- a release (or advisory) is published, or
- maintainers request coordinated disclosure

Out of scope (generally):

- Social engineering
- Denial-of-service requiring unrealistic resources
- Vulnerabilities in third-party services outside the project control (unless caused by SIGNIA integration)

If you are unsure whether an issue is in scope, report it anyway.

---

## Response and disclosure timeline

We aim to follow these targets:

- **Acknowledgement:** within 72 hours
- **Triage / confirmation:** 7 days
- **Fix development:** depends on severity and complexity
- **Coordinated disclosure:** mutually agreed schedule

For critical issues with active exploitation risk, we may ship an emergency fix and advisory as soon as reasonably possible.

---

## Severity guidance

We use a simplified severity rubric:

- **Critical:** remote compromise, key theft, registry authority takeover, arbitrary instruction execution, etc.
- **High:** auth bypass, significant data exposure, economic exploits affecting registry correctness
- **Medium:** limited disclosure, constrained bypasses, non-trivial exploit prerequisites
- **Low:** minor info leaks, hard-to-exploit issues, missing hardening

Severity is determined by maintainers during triage.

---

## Safe harbor

We support good-faith security research.

If you:

- make a good-faith effort to avoid privacy violations and destructive actions
- only use accounts/data you own or have explicit permission to test
- report vulnerabilities promptly and privately
- do not publicly disclose before coordination

…we will not pursue legal action against you for your research.

---

## Handling secrets

Do not include secrets (private keys, API tokens, production credentials) in reports.

If you accidentally expose a secret in a report, notify maintainers immediately so we can rotate and redact.

---

## Supply-chain security

We aim to maintain a hardened supply chain:

- Dependabot updates for Rust and Node dependencies
- CodeQL scanning
- `cargo deny` / `cargo audit` checks
- Lockfiles enforced (Cargo.lock where applicable, pnpm-lock.yaml)
- Minimal CI permissions by default

If you suspect a compromised dependency, report it as **Critical**.

---

## Contact

- Preferred: GitHub Security Advisories
- Alternative: `security@signia.example` (placeholder)

Thank you for helping keep SIGNIA safe.
