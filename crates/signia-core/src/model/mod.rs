//! SIGNIA data models.
//!
//! This module defines the strongly-typed Rust representations for SIGNIA artifact
//! formats. It is intentionally versioned: each wire format is isolated under a
//! `vN` module so that upgrades can be introduced without breaking existing users.
//!
//! Design goals:
//! - **Version isolation:** `v1` types never change in breaking ways. New formats go in `v2`, etc.
//! - **Deterministic serialization:** the canonical bytes used for hashing are produced by
//!   `crate::canonical` (not by default serde formatting).
//! - **Minimal policy:** models are mostly "dumb" data. Higher layers (CLI/API) apply policy,
//!   validation, limits, and I/O.
//!
//! Recommended imports:
//! - For most code: `use signia_core::model::v1::*;`
//! - Or: `use signia_core::prelude::*;`
//!
//! Notes on serde:
//! - These models are intended to be serde-friendly when the `canonical-json` feature is enabled.
//! - Canonical hashing must never rely on default `serde_json::to_vec` because it is not a
//!   canonical form (ordering/whitespace are not guaranteed). Use `crate::canonical` instead.

// pub mod v1;

pub use v1::{
    EdgeV1, EntityV1, ManifestV1, ProofV1, SchemaV1,
    // Supporting structures
    BundleInfoV1, HashRefV1, HashSpecV1, InputRefV1, LimitsV1, NormalizationV1, OutputFileV1,
    OutputStatsV1, PluginRefV1, SourceRefV1,
};

/// A versioned schema enum for ergonomic APIs that want to accept multiple versions.
///
/// Higher layers can match on this enum to implement backward/forward compatible handling.
#[derive(Debug, Clone)]
pub enum AnySchema {
    V1(SchemaV1),
}

/// A versioned manifest enum for ergonomic APIs that want to accept multiple versions.
#[derive(Debug, Clone)]
pub enum AnyManifest {
    V1(ManifestV1),
}

/// A versioned proof enum for ergonomic APIs that want to accept multiple versions.
#[derive(Debug, Clone)]
pub enum AnyProof {
    V1(ProofV1),
}

impl AnySchema {
    /// Return the schema version string.
    pub fn version(&self) -> &'static str {
        match self {
            AnySchema::V1(_) => "v1",
        }
    }
}

impl AnyManifest {
    /// Return the manifest version string.
    pub fn version(&self) -> &'static str {
        match self {
            AnyManifest::V1(_) => "v1",
        }
    }
}

impl AnyProof {
    /// Return the proof version string.
    pub fn version(&self) -> &'static str {
        match self {
            AnyProof::V1(_) => "v1",
        }
    }
}

/// Lightweight validation helpers for model consumers.
///
/// These checks are intentionally minimal and do not enforce compilation-time policies.
/// Full verification is performed by `signia verify` in the CLI / API.
///
/// If you need strict verification, use:
/// - recompute canonical hashes via `crate::canonical` + `crate::hash`
/// - recompute Merkle roots via `crate::merkle`
pub mod validate {
    use super::*;
    use crate::errors::{SigniaError, SigniaResult};

    /// Validate basic structural invariants for a v1 schema instance.
    ///
    /// This does **not** guarantee determinism correctness. It checks:
    /// - required top-level fields are present (enforced by type)
    /// - entity ids are unique
    /// - edge ids are unique
    /// - edge endpoints reference existing entity ids (best-effort)
    pub fn schema_v1_basic(schema: &SchemaV1) -> SigniaResult<()> {
        use std::collections::HashSet;

        let mut ent_ids = HashSet::new();
        for e in &schema.entities {
            if !ent_ids.insert(e.id.clone()) {
                return Err(SigniaError::invalid_argument(format!(
                    "duplicate entity id: {}",
                    e.id
                )));
            }
        }

        let mut edge_ids = HashSet::new();
        for ed in &schema.edges {
            if !edge_ids.insert(ed.id.clone()) {
                return Err(SigniaError::invalid_argument(format!(
                    "duplicate edge id: {}",
                    ed.id
                )));
            }
            if !ent_ids.contains(&ed.from) {
                return Err(SigniaError::invalid_argument(format!(
                    "edge {} references missing from-entity id: {}",
                    ed.id, ed.from
                )));
            }
            if !ent_ids.contains(&ed.to) {
                return Err(SigniaError::invalid_argument(format!(
                    "edge {} references missing to-entity id: {}",
                    ed.id, ed.to
                )));
            }
        }

        Ok(())
    }
}
