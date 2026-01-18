//! Normalization utilities for the built-in API layer.
//!
//! This module contains deterministic normalization helpers used by hosts
//! to validate and normalize user inputs before they are passed into plugins.
//!
//! This is a "pure utilities" module:
//! - no filesystem I/O
//! - no network I/O
//! - no time-based behavior
//!
//! Typical usage:
//! - normalize plugin id ("builtin.repo")
//! - normalize input type ("repo")
//! - normalize artifact kind ("schema", "manifest", "proof", "ir")
//! - normalize small JSON payloads (canonical ordering) before hashing

#![cfg(feature = "builtin")]

use anyhow::{anyhow, Result};
use serde_json::Value;

use signia_core::determinism::canonical_json::canonicalize_json;

/// Normalize a plugin id.
///
/// Rules:
/// - trim whitespace
/// - must be ASCII and match `[a-z0-9][a-z0-9._-]{1,63}`
/// - no leading/trailing dots
pub fn normalize_plugin_id(id: &str) -> Result<String> {
    let s = id.trim();
    if s.len() < 2 || s.len() > 64 {
        return Err(anyhow!("plugin id must be 2..64 chars"));
    }
    if s.starts_with('.') || s.ends_with('.') {
        return Err(anyhow!("plugin id must not start/end with '.'"));
    }
    if !s.is_ascii() {
        return Err(anyhow!("plugin id must be ASCII"));
    }
    let bytes = s.as_bytes();
    if !is_id_start(bytes[0]) {
        return Err(anyhow!("plugin id must start with [a-z0-9]"));
    }
    for &b in bytes {
        if !(b.is_ascii_lowercase()
            || b.is_ascii_digit()
            || b == b'.'
            || b == b'_'
            || b == b'-')
        {
            return Err(anyhow!("plugin id contains invalid character"));
        }
    }
    Ok(s.to_string())
}

fn is_id_start(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit()
}

/// Normalize an input type (e.g. "repo", "dataset").
///
/// Rules:
/// - lowercase
/// - trim whitespace
/// - must be ASCII and match `[a-z][a-z0-9_-]{1,31}`
pub fn normalize_input_type(t: &str) -> Result<String> {
    let s = t.trim().to_ascii_lowercase();
    if s.len() < 2 || s.len() > 32 {
        return Err(anyhow!("input type must be 2..32 chars"));
    }
    if !s.is_ascii() {
        return Err(anyhow!("input type must be ASCII"));
    }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return Err(anyhow!("input type must start with [a-z]"));
    }
    for &b in bytes {
        if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-') {
            return Err(anyhow!("input type contains invalid character"));
        }
    }
    Ok(s)
}

/// Normalize an artifact kind.
///
/// Supported:
/// - schema
/// - manifest
/// - proof
/// - ir
/// - fingerprint
pub fn normalize_artifact_kind(kind: &str) -> Result<String> {
    let s = kind.trim().to_ascii_lowercase();
    let ok = matches!(s.as_str(), "schema" | "manifest" | "proof" | "ir" | "fingerprint");
    if !ok {
        return Err(anyhow!("unsupported artifact kind: {kind}"));
    }
    Ok(s)
}

/// Canonicalize a JSON value deterministically.
///
/// This is a thin wrapper over `signia_core::determinism::canonical_json`.
pub fn canonicalize_json_value(v: &Value) -> Result<Value> {
    canonicalize_json(v)
}

/// Canonicalize and serialize a JSON value to bytes deterministically.
pub fn canonical_json_bytes(v: &Value) -> Result<Vec<u8>> {
    let c = canonicalize_json(v)?;
    // serde_json serialization is deterministic for canonicalized structure
    // (sorted object keys, stable arrays).
    let bytes = serde_json::to_vec(&c)?;
    Ok(bytes)
}

/// Normalize a small user-provided JSON payload before hashing.
///
/// This is useful for caching and reproducibility.
pub fn normalize_payload_for_hashing(v: &Value) -> Result<Vec<u8>> {
    canonical_json_bytes(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_plugin_id() {
        assert_eq!(normalize_plugin_id(" builtin.repo ").unwrap(), "builtin.repo");
        assert!(normalize_plugin_id("Bad").is_err()); // uppercase
    }

    #[test]
    fn normalizes_input_type() {
        assert_eq!(normalize_input_type(" Repo ").unwrap(), "repo");
        assert!(normalize_input_type("r").is_err());
    }

    #[test]
    fn artifact_kind_validation() {
        assert_eq!(normalize_artifact_kind("Schema").unwrap(), "schema");
        assert!(normalize_artifact_kind("binary").is_err());
    }

    #[test]
    fn canonical_json_sorts_keys() {
        let v = json!({"b":1,"a":2});
        let bytes = canonical_json_bytes(&v).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.find(r#""a":2"#).unwrap() < s.find(r#""b":1"#).unwrap());
    }
}
