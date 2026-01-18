//! Dataset checksums and fingerprints for the built-in `dataset` plugin.
//!
//! This module provides deterministic, auditable hashing utilities used by the host
//! and the plugin pipeline to derive stable dataset fingerprints.
//!
//! IMPORTANT:
//! - This code performs no filesystem or network I/O.
//! - The host provides file bytes (or content hashes) for the files to include.
//! - Hashing is deterministic: stable ordering, stable normalization rules.
//!
//! Supported outputs:
//! - per-file sha256 hex
//! - dataset fingerprint hash over (path, size, sha256) tuples
//! - optional Merkle root for large datasets (path-keyed)

#![cfg(feature = "builtin")]

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use signia_core::determinism::hashing::hash_bytes_hex;
use signia_core::determinism::merkle::{merkle_root_hex, MerkleLeaf};

use crate::builtin::repo::tree_walk::normalize_repo_path;

/// A host-provided dataset file record used for checksum computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetFileRecord {
    pub path: String,
    pub size: u64,
    /// Optional file bytes. If present, sha256 will be computed from bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    /// Optional precomputed sha256 hex. If bytes is absent, this must be provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl DatasetFileRecord {
    pub fn new(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            size,
            bytes: None,
            sha256: None,
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.size = bytes.len() as u64;
        self.bytes = Some(bytes);
        self
    }

    pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
        self.sha256 = Some(sha256.into());
        self
    }
}

/// Compute sha256 hex for a record deterministically.
///
/// If bytes are present, sha256 is computed from bytes.
/// Otherwise, sha256 must be provided.
pub fn ensure_file_sha256(record: &mut DatasetFileRecord) -> Result<()> {
    if let Some(b) = &record.bytes {
        record.sha256 = Some(hash_bytes_hex(b)?);
        record.size = b.len() as u64;
        return Ok(());
    }
    if record.sha256.is_none() {
        return Err(anyhow!(
            "missing sha256 for file without bytes: {}",
            record.path
        ));
    }
    Ok(())
}

/// Compute per-file sha256 map keyed by normalized path.
///
/// Returns: BTreeMap<path, sha256>
pub fn compute_checksums(mut files: Vec<DatasetFileRecord>) -> Result<BTreeMap<String, String>> {
    // Normalize paths and compute sha256.
    let mut out: BTreeMap<String, String> = BTreeMap::new();

    // Deterministic order: sort by normalized path.
    files.sort_by(|a, b| a.path.cmp(&b.path));

    for mut f in files {
        let p = normalize_repo_path(&f.path)?;
        ensure_file_sha256(&mut f)?;
        out.insert(p, f.sha256.clone().unwrap());
    }

    Ok(out)
}

/// Compute a stable dataset fingerprint:
/// sha256( concat( path \t size \t sha256 \n ) sorted by path )
pub fn dataset_fingerprint(mut files: Vec<DatasetFileRecord>) -> Result<String> {
    // Normalize, compute sha256, then sort by normalized path.
    for f in &mut files {
        f.path = normalize_repo_path(&f.path)?;
        ensure_file_sha256(f)?;
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut buf = Vec::new();
    for f in &files {
        buf.extend_from_slice(f.path.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(f.size.to_string().as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(f.sha256.as_ref().unwrap().as_bytes());
        buf.extend_from_slice(b"\n");
    }

    hash_bytes_hex(&buf)
}

/// Compute a deterministic Merkle root over dataset files.
///
/// Leaves are keyed by normalized path:
/// leaf = sha256( path \n sha256 \n size )
///
/// This is useful when you want to prove inclusion of a file without
/// including the entire fingerprint list.
pub fn dataset_merkle_root(mut files: Vec<DatasetFileRecord>) -> Result<String> {
    for f in &mut files {
        f.path = normalize_repo_path(&f.path)?;
        ensure_file_sha256(f)?;
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let leaves: Vec<MerkleLeaf> = files
        .iter()
        .map(|f| {
            let mut buf = Vec::new();
            buf.extend_from_slice(f.path.as_bytes());
            buf.extend_from_slice(b"\n");
            buf.extend_from_slice(f.sha256.as_ref().unwrap().as_bytes());
            buf.extend_from_slice(b"\n");
            buf.extend_from_slice(f.size.to_string().as_bytes());
            MerkleLeaf { key: f.path.clone(), value: buf }
        })
        .collect();

    merkle_root_hex(&leaves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_stable() {
        let a = DatasetFileRecord::new("b.txt", 1).with_bytes(b"b".to_vec());
        let b = DatasetFileRecord::new("a.txt", 1).with_bytes(b"a".to_vec());

        let f1 = dataset_fingerprint(vec![a.clone(), b.clone()]).unwrap();
        let f2 = dataset_fingerprint(vec![b, a]).unwrap();
        assert_eq!(f1, f2);
    }

    #[test]
    fn merkle_root_stable() {
        let a = DatasetFileRecord::new("x.txt", 1).with_bytes(b"x".to_vec());
        let b = DatasetFileRecord::new("y.txt", 1).with_bytes(b"y".to_vec());
        let r1 = dataset_merkle_root(vec![a.clone(), b.clone()]).unwrap();
        let r2 = dataset_merkle_root(vec![b, a]).unwrap();
        assert_eq!(r1, r2);
    }
}
