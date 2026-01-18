//! Parsing helpers for SIGNIA pipeline inputs.
//!
//! Core crate avoids filesystem/network I/O. Parsing helpers in this module operate on:
//! - in-memory bytes
//! - in-memory JSON values
//! - already-validated decoded structures
//!
//! This module provides:
//! - strict JSON parsing with size limits
//! - format detection (schema/manifest/proof)
//! - version dispatch (currently v1)
//! - helpful error messages for API/CLI consumers
//!
//! Determinism note:
//! - parsing is deterministic given the same bytes
//! - callers should provide explicit limits, not rely on environment

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use crate::model::v1::{ManifestV1, ProofV1, SchemaV1};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Default maximum JSON bytes accepted by helpers (2 MiB).
pub const DEFAULT_MAX_JSON_BYTES: usize = 2 * 1024 * 1024;

/// Input format classification for SIGNIA artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Schema,
    Manifest,
    Proof,
    Unknown,
}

/// Parse JSON bytes into `serde_json::Value` with a hard size limit.
#[cfg(feature = "canonical-json")]
pub fn parse_json_bytes(bytes: &[u8], max_bytes: usize) -> SigniaResult<Value> {
    if bytes.len() > max_bytes {
        return Err(SigniaError::invalid_argument(format!(
            "JSON payload too large ({} bytes > limit {})",
            bytes.len(),
            max_bytes
        )));
    }

    serde_json::from_slice(bytes)
        .map_err(|e| SigniaError::serialization(format!("failed to parse JSON: {e}")))
}

/// Detect artifact kind from parsed JSON.
///
/// We use lightweight heuristics:
/// - schema: has `version`, `kind`, `entities`, `edges`
/// - manifest: has `version`, `schemas`, `inputs`, `outputs`, `limits`
/// - proof: has `version`, `hashAlg`, `root`, `leaves`
#[cfg(feature = "canonical-json")]
pub fn detect_kind(v: &Value) -> ArtifactKind {
    let obj = match v.as_object() {
        Some(o) => o,
        None => return ArtifactKind::Unknown,
    };

    let has_version = obj.get("version").and_then(|x| x.as_str()).is_some();

    if !has_version {
        return ArtifactKind::Unknown;
    }

    let is_schema = obj.contains_key("kind") && obj.contains_key("entities") && obj.contains_key("edges");
    if is_schema {
        return ArtifactKind::Schema;
    }

    let is_manifest = obj.contains_key("schemas") && obj.contains_key("inputs") && obj.contains_key("outputs") && obj.contains_key("limits");
    if is_manifest {
        return ArtifactKind::Manifest;
    }

    let is_proof = obj.contains_key("hashAlg") && obj.contains_key("root") && obj.contains_key("leaves");
    if is_proof {
        return ArtifactKind::Proof;
    }

    ArtifactKind::Unknown
}

/// Read the `version` field from a JSON object.
#[cfg(feature = "canonical-json")]
pub fn read_version(v: &Value) -> SigniaResult<String> {
    let obj = v.as_object().ok_or_else(|| SigniaError::invalid_argument("expected JSON object"))?;
    let ver = obj
        .get("version")
        .and_then(|x| x.as_str())
        .ok_or_else(|| SigniaError::invalid_argument("missing version string"))?;
    Ok(ver.to_string())
}

/// Parse bytes into a SchemaV1.
#[cfg(feature = "canonical-json")]
pub fn parse_schema_v1(bytes: &[u8], max_bytes: usize) -> SigniaResult<SchemaV1> {
    let v = parse_json_bytes(bytes, max_bytes)?;
    let kind = detect_kind(&v);
    if kind != ArtifactKind::Schema {
        return Err(SigniaError::invalid_argument("input is not a schema"));
    }
    let ver = read_version(&v)?;
    if ver != "v1" {
        return Err(SigniaError::invalid_argument(format!("unsupported schema version: {ver}")));
    }
    serde_json::from_value(v).map_err(|e| SigniaError::serialization(format!("failed to decode SchemaV1: {e}")))
}

/// Parse bytes into a ManifestV1.
#[cfg(feature = "canonical-json")]
pub fn parse_manifest_v1(bytes: &[u8], max_bytes: usize) -> SigniaResult<ManifestV1> {
    let v = parse_json_bytes(bytes, max_bytes)?;
    let kind = detect_kind(&v);
    if kind != ArtifactKind::Manifest {
        return Err(SigniaError::invalid_argument("input is not a manifest"));
    }
    let ver = read_version(&v)?;
    if ver != "v1" {
        return Err(SigniaError::invalid_argument(format!("unsupported manifest version: {ver}")));
    }
    serde_json::from_value(v).map_err(|e| SigniaError::serialization(format!("failed to decode ManifestV1: {e}")))
}

/// Parse bytes into a ProofV1.
#[cfg(feature = "canonical-json")]
pub fn parse_proof_v1(bytes: &[u8], max_bytes: usize) -> SigniaResult<ProofV1> {
    let v = parse_json_bytes(bytes, max_bytes)?;
    let kind = detect_kind(&v);
    if kind != ArtifactKind::Proof {
        return Err(SigniaError::invalid_argument("input is not a proof"));
    }
    let ver = read_version(&v)?;
    if ver != "v1" {
        return Err(SigniaError::invalid_argument(format!("unsupported proof version: {ver}")));
    }
    serde_json::from_value(v).map_err(|e| SigniaError::serialization(format!("failed to decode ProofV1: {e}")))
}

/// Parse any artifact and return (kind, json, version).
#[cfg(feature = "canonical-json")]
pub fn parse_any(bytes: &[u8], max_bytes: usize) -> SigniaResult<(ArtifactKind, Value, String)> {
    let v = parse_json_bytes(bytes, max_bytes)?;
    let kind = detect_kind(&v);
    let ver = read_version(&v)?;
    Ok((kind, v, ver))
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;

    #[test]
    fn detect_schema() {
        let v = serde_json::json!({
            "version":"v1",
            "kind":"repo",
            "meta":{},
            "entities":[],
            "edges":[]
        });
        assert_eq!(detect_kind(&v), ArtifactKind::Schema);
    }

    #[test]
    fn detect_manifest() {
        let v = serde_json::json!({
            "version":"v1",
            "name":"x",
            "schemas":[],
            "inputs":[],
            "outputs":[],
            "plugins":[],
            "limits":{"maxFiles":1,"maxBytes":1,"maxNodes":1,"maxEdges":1,"timeoutMs":1,"network":"deny"}
        });
        assert_eq!(detect_kind(&v), ArtifactKind::Manifest);
    }

    #[test]
    fn detect_proof() {
        let v = serde_json::json!({
            "version":"v1",
            "hashAlg":"sha256",
            "root":"a",
            "leaves":[]
        });
        assert_eq!(detect_kind(&v), ArtifactKind::Proof);
    }

    #[test]
    fn parse_json_bytes_respects_limit() {
        let bytes = br#"{"version":"v1"}"#;
        let v = parse_json_bytes(bytes, 1024).unwrap();
        assert_eq!(v["version"], "v1");

        let err = parse_json_bytes(bytes, 1).err().unwrap();
        let s = err.to_string();
        assert!(s.contains("too large"));
    }
}
