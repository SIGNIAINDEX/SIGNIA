//! Deterministic hashing utilities for SIGNIA.
//!
//! This module defines all hashing primitives used across SIGNIA.
//! All hashes are:
//! - deterministic
//! - domain-separated
//! - explicitly parameterized
//!
//! Supported algorithms:
//! - sha256
//!
//! No implicit defaults are allowed. Callers must choose algorithms explicitly.

use crate::errors::{SigniaError, SigniaResult};

use sha2::{Digest, Sha256};

/// Hash algorithm identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashAlg {
    Sha256,
}

impl HashAlg {
    pub fn from_str(s: &str) -> SigniaResult<Self> {
        match s {
            "sha256" => Ok(HashAlg::Sha256),
            _ => Err(SigniaError::invalid_argument(format!(
                "unsupported hash algorithm: {s}"
            ))),
        }
    }
}

/// Hash raw bytes using the selected algorithm.
pub fn hash_bytes(alg: HashAlg, bytes: &[u8]) -> Vec<u8> {
    match alg {
        HashAlg::Sha256 => {
            let mut h = Sha256::new();
            h.update(bytes);
            h.finalize().to_vec()
        }
    }
}

/// Hash raw bytes and return lowercase hex string.
pub fn hash_bytes_hex(bytes: &[u8]) -> SigniaResult<String> {
    let h = hash_bytes(HashAlg::Sha256, bytes);
    Ok(hex::encode(h))
}

/// Domain-separated Merkle leaf hash.
pub fn hash_merkle_leaf_hex(alg: &str, payload: &[u8]) -> SigniaResult<String> {
    let alg = HashAlg::from_str(alg)?;
    let mut buf = Vec::new();
    buf.extend_from_slice(crate::domain::MERKLE_LEAF.as_bytes());
    buf.extend_from_slice(payload);
    Ok(hex::encode(hash_bytes(alg, &buf)))
}

/// Domain-separated Merkle internal node hash.
pub fn hash_merkle_node_hex(alg: &str, left_hex: &str, right_hex: &str) -> SigniaResult<String> {
    let alg = HashAlg::from_str(alg)?;
    let left = hex::decode(left_hex)
        .map_err(|_| SigniaError::invalid_argument("invalid left hex"))?;
    let right = hex::decode(right_hex)
        .map_err(|_| SigniaError::invalid_argument("invalid right hex"))?;

    let mut buf = Vec::new();
    buf.extend_from_slice(crate::domain::MERKLE_NODE.as_bytes());
    buf.extend_from_slice(&left);
    buf.extend_from_slice(&right);

    Ok(hex::encode(hash_bytes(alg, &buf)))
}

#[cfg(feature = "canonical-json")]
use crate::determinism::canonical_json;

/// Hash canonical JSON value.
#[cfg(feature = "canonical-json")]
pub fn hash_canonical_json_hex(value: &serde_json::Value) -> SigniaResult<String> {
    let bytes = canonical_json::to_canonical_bytes(value)?;
    Ok(hex::encode(hash_bytes(HashAlg::Sha256, &bytes)))
}

/// Hash SchemaV1.
#[cfg(feature = "canonical-json")]
pub fn hash_schema_v1_hex(schema: &crate::model::v1::SchemaV1) -> SigniaResult<String> {
    hash_canonical_json_hex(&serde_json::to_value(schema).map_err(|e| {
        SigniaError::serialization(format!("failed to serialize schema: {e}"))
    })?)
}

/// Hash ManifestV1.
#[cfg(feature = "canonical-json")]
pub fn hash_manifest_v1_hex(manifest: &crate::model::v1::ManifestV1) -> SigniaResult<String> {
    hash_canonical_json_hex(&serde_json::to_value(manifest).map_err(|e| {
        SigniaError::serialization(format!("failed to serialize manifest: {e}"))
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_bytes_stable() {
        let h1 = hash_bytes_hex(b"abc").unwrap();
        let h2 = hash_bytes_hex(b"abc").unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn merkle_leaf_and_node() {
        let leaf = hash_merkle_leaf_hex("sha256", b"x").unwrap();
        let node = hash_merkle_node_hex("sha256", &leaf, &leaf).unwrap();
        assert!(!node.is_empty());
    }
}
