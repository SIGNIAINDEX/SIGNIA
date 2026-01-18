//! signia-core
//!
//! Core primitives for SIGNIA:
//! - Schema/Manifest/Proof models (v1)
//! - Canonical JSON encoding for deterministic hashing
//! - Hashing utilities with domain separation
//! - Merkle tree roots and inclusion proofs
//! - Artifact path normalization helpers

pub mod canonical;
pub mod errors;
pub mod hash;
pub mod merkle;
pub mod model;
pub mod path;

pub use crate::errors::{SigniaError, SigniaResult};

/// Common version strings.
pub const SCHEMA_VERSION_V1: &str = "v1";

/// Default domain separation labels.
/// These must remain stable across versions.
pub mod domain {
    pub const SCHEMA: &str = "signia.v1.schema";
    pub const MANIFEST: &str = "signia.v1.manifest";
    pub const PROOF: &str = "signia.v1.proof";
    pub const MERKLE_LEAF: &str = "signia.v1.merkle.leaf";
    pub const MERKLE_NODE: &str = "signia.v1.merkle.node";
}

/// Default canonicalization settings.
pub mod defaults {
    /// Canonical newline normalization.
    pub const NEWLINE: &str = "lf";
    /// Canonical encoding.
    pub const ENCODING: &str = "utf-8";
    /// Canonical artifact path root.
    pub const PATH_ROOT: &str = "artifact:/";
}

/// Convenience re-exports.
pub mod prelude {
    pub use crate::canonical::{canonical_json_bytes, canonical_json_value, CanonicalJsonOptions};
    pub use crate::hash::{hash_bytes, hash_with_domain, HashAlg, HashDigest};
    pub use crate::merkle::{MerkleProof, MerkleTree, MerkleTreeOptions};
    pub use crate::model::v1::{EdgeV1, EntityV1, ManifestV1, ProofV1, SchemaV1};
    pub use crate::path::{ArtifactPath, ArtifactPathError, PathPolicy};
    pub use crate::{SigniaError, SigniaResult};
}
