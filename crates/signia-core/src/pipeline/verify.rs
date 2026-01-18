//! Verification orchestration for SIGNIA bundles.
//!
//! This module provides deterministic verification routines for SIGNIA artifacts.
//! It is intended to be used by:
//! - CLI (`signia verify ...`)
//! - API service (`POST /v1/verify`)
//! - CI jobs verifying outputs
//!
//! Core verifications include:
//! - schema structural validation
//! - manifest structural validation
//! - canonical hashing checks (schemaHash, manifestHash)
//! - proof Merkle root reconstruction and leaf checks
//! - optional inclusion proof checks (if provided)
//!
//! Security note:
//! - This module performs no I/O.
//! - All verification operates on in-memory structures.
//! - Hashing uses domain-separated functions (crate::hash + crate::domain).

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use crate::model::v1::{InclusionProofV1, LeafV1, ManifestV1, ProofV1, SchemaV1, SiblingV1};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

/// Verification input bundle.
#[derive(Debug, Clone)]
pub struct VerifyBundle {
    #[cfg(feature = "canonical-json")]
    pub schema: SchemaV1,
    #[cfg(feature = "canonical-json")]
    pub manifest: ManifestV1,
    #[cfg(feature = "canonical-json")]
    pub proof: Option<ProofV1>,
}

/// Verification options.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// If true, require a proof and validate its root.
    pub require_proof: bool,

    /// If true, validate any inclusion proofs present.
    pub validate_inclusions: bool,

    /// If true, require manifest.schemas include the schema digest.
    pub require_manifest_binding: bool,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            require_proof: true,
            validate_inclusions: true,
            require_manifest_binding: true,
        }
    }
}

/// A structured verification finding.
#[derive(Debug, Clone)]
pub struct VerifyFinding {
    pub level: VerifyLevel,
    pub code: String,
    pub message: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum VerifyLevel {
    Info,
    Warning,
    Error,
}

/// Verification report.
#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub ok: bool,
    pub findings: Vec<VerifyFinding>,
    pub schema_hash_hex: Option<String>,
    pub manifest_hash_hex: Option<String>,
    pub proof_root_hex: Option<String>,
}

impl VerifyReport {
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|f| matches!(f.level, VerifyLevel::Error))
    }
}

fn push(
    findings: &mut Vec<VerifyFinding>,
    level: VerifyLevel,
    code: impl Into<String>,
    message: impl Into<String>,
) {
    findings.push(VerifyFinding {
        level,
        code: code.into(),
        message: message.into(),
        data: BTreeMap::new(),
    });
}

/// Verify a bundle deterministically.
///
/// Returns a report even if verification fails (for UI). Use `report.ok` or `report.has_errors()`.
#[cfg(feature = "canonical-json")]
pub fn verify_bundle(bundle: VerifyBundle, opts: VerifyOptions) -> SigniaResult<VerifyReport> {
    let mut findings = Vec::new();

    // 1) Structural validation
    verify_schema_structure(&bundle.schema, &mut findings)?;
    verify_manifest_structure(&bundle.manifest, &mut findings)?;

    // 2) Canonical hashes
    let schema_hash = crate::hash::hash_schema_v1_hex(&bundle.schema)?;
    let manifest_hash = crate::hash::hash_manifest_v1_hex(&bundle.manifest)?;

    push(
        &mut findings,
        VerifyLevel::Info,
        "hash.schema",
        format!("schema hash computed: {}", &schema_hash),
    );
    push(
        &mut findings,
        VerifyLevel::Info,
        "hash.manifest",
        format!("manifest hash computed: {}", &manifest_hash),
    );

    // 3) Manifest binding
    if opts.require_manifest_binding {
        let mut found = false;
        for s in &bundle.manifest.schemas {
            if s.digest == schema_hash {
                found = true;
                break;
            }
        }
        if !found {
            push(
                &mut findings,
                VerifyLevel::Error,
                "manifest.binding.missing",
                "manifest.schemas does not contain schema digest",
            );
        }
    }

    // 4) Proof
    let mut proof_root = None;
    if opts.require_proof && bundle.proof.is_none() {
        push(
            &mut findings,
            VerifyLevel::Error,
            "proof.missing",
            "proof is required but not provided",
        );
    }

    if let Some(p) = &bundle.proof {
        // Leaves must include schemaHash and manifestHash
        let mut leaf_map: BTreeMap<String, String> = BTreeMap::new();
        for l in &p.leaves {
            leaf_map.insert(l.key.clone(), l.value.clone());
        }

        if leaf_map.get("digest:schemaHash") != Some(&schema_hash) {
            push(
                &mut findings,
                VerifyLevel::Error,
                "proof.leaf.schemaHash.mismatch",
                "proof leaf digest:schemaHash does not match computed schema hash",
            );
        }

        if leaf_map.get("digest:manifestHash") != Some(&manifest_hash) {
            push(
                &mut findings,
                VerifyLevel::Error,
                "proof.leaf.manifestHash.mismatch",
                "proof leaf digest:manifestHash does not match computed manifest hash",
            );
        }

        // Recompute root
        let root = recompute_proof_root_hex(p)?;
        proof_root = Some(root.clone());

        if root != p.root {
            push(
                &mut findings,
                VerifyLevel::Error,
                "proof.root.mismatch",
                "recomputed proof root does not match provided root",
            );
        } else {
            push(
                &mut findings,
                VerifyLevel::Info,
                "proof.root.ok",
                "proof root matches",
            );
        }

        if opts.validate_inclusions {
            if let Some(incs) = &p.inclusions {
                for inc in incs {
                    if let Err(e) = verify_inclusion(p, inc) {
                        push(
                            &mut findings,
                            VerifyLevel::Error,
                            "proof.inclusion.invalid",
                            format!("inclusion proof invalid for {}: {}", inc.key, e),
                        );
                    }
                }
            }
        }
    }

    let ok = !findings.iter().any(|f| matches!(f.level, VerifyLevel::Error));

    Ok(VerifyReport {
        ok,
        findings,
        schema_hash_hex: Some(schema_hash),
        manifest_hash_hex: Some(manifest_hash),
        proof_root_hex: proof_root,
    })
}

/// Basic schema structure checks.
#[cfg(feature = "canonical-json")]
fn verify_schema_structure(schema: &SchemaV1, findings: &mut Vec<VerifyFinding>) -> SigniaResult<()> {
    if schema.version != "v1" {
        push(
            findings,
            VerifyLevel::Error,
            "schema.version",
            format!("unsupported schema version: {}", schema.version),
        );
    }

    if schema.kind.trim().is_empty() {
        push(findings, VerifyLevel::Error, "schema.kind", "schema.kind is empty");
    }

    // Require meta object
    if !schema.meta.is_object() {
        push(findings, VerifyLevel::Error, "schema.meta", "schema.meta must be an object");
    } else {
        // Minimal required keys
        let obj = schema.meta.as_object().unwrap();
        for k in ["name", "createdAt", "source", "normalization"] {
            if !obj.contains_key(k) {
                push(
                    findings,
                    VerifyLevel::Error,
                    "schema.meta.missing",
                    format!("schema.meta missing required key: {k}"),
                );
            }
        }
    }

    // Ensure entity ids are unique and non-empty
    let mut ids = BTreeMap::<String, ()>::new();
    for e in &schema.entities {
        if e.id.trim().is_empty() {
            push(findings, VerifyLevel::Error, "schema.entity.id", "entity id is empty");
        }
        if ids.insert(e.id.clone(), ()).is_some() {
            push(
                findings,
                VerifyLevel::Error,
                "schema.entity.id.duplicate",
                "duplicate entity id",
            );
        }
        if e.r#type.trim().is_empty() {
            push(findings, VerifyLevel::Error, "schema.entity.type", "entity type is empty");
        }
    }

    // Validate edge refs
    for ed in &schema.edges {
        if ed.from.trim().is_empty() || ed.to.trim().is_empty() {
            push(findings, VerifyLevel::Error, "schema.edge.refs", "edge refs empty");
        }
        if !ids.contains_key(&ed.from) {
            push(
                findings,
                VerifyLevel::Error,
                "schema.edge.from.unknown",
                "edge.from refers to unknown entity",
            );
        }
        if !ids.contains_key(&ed.to) {
            push(
                findings,
                VerifyLevel::Error,
                "schema.edge.to.unknown",
                "edge.to refers to unknown entity",
            );
        }
    }

    Ok(())
}

/// Basic manifest structure checks.
#[cfg(feature = "canonical-json")]
fn verify_manifest_structure(manifest: &ManifestV1, findings: &mut Vec<VerifyFinding>) -> SigniaResult<()> {
    if manifest.version != "v1" {
        push(
            findings,
            VerifyLevel::Error,
            "manifest.version",
            format!("unsupported manifest version: {}", manifest.version),
        );
    }

    if manifest.name.trim().is_empty() {
        push(findings, VerifyLevel::Error, "manifest.name", "manifest.name is empty");
    }

    // Limits sanity
    if manifest.limits.max_files == 0 {
        push(findings, VerifyLevel::Warning, "manifest.limits.maxFiles", "maxFiles is 0");
    }
    if manifest.limits.timeout_ms == 0 {
        push(findings, VerifyLevel::Warning, "manifest.limits.timeoutMs", "timeoutMs is 0");
    }

    Ok(())
}

/// Recompute a proof root from its leaves.
///
/// This matches the construction in `pipeline::compile` and `pipeline::stages::BuildProofV1Stage`:
/// - leaf payload: "key=value"
/// - leaf hash: domain-separated using merkle tree options
#[cfg(feature = "canonical-json")]
pub fn recompute_proof_root_hex(proof: &ProofV1) -> SigniaResult<String> {
    let mut leaves = proof.leaves.clone();
    leaves.sort_by(|a, b| a.key.cmp(&b.key));

    let mut tree = crate::merkle::MerkleTree::new(crate::merkle::MerkleTreeOptions {
        hash_alg: proof.hash_alg.clone(),
        domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
        domain_node: crate::domain::MERKLE_NODE.to_string(),
    });

    for leaf in &leaves {
        let payload = format!("{}={}", leaf.key, leaf.value);
        tree.push_leaf(payload.as_bytes())?;
    }

    tree.root_hex()
}

/// Verify a single inclusion proof.
///
/// Inclusion verification reconstructs the root by applying siblings in order.
/// Each sibling specifies a side ("left" or "right").
///
/// Hashing matches the Merkle tree hashing:
/// - leaf hash is hash(domain_leaf || payload)
/// - internal node hash is hash(domain_node || left || right)
#[cfg(feature = "canonical-json")]
pub fn verify_inclusion(proof: &ProofV1, inc: &InclusionProofV1) -> SigniaResult<()> {
    // Ensure the leaf exists in proof.leaves
    let mut found = false;
    for l in &proof.leaves {
        if l.key == inc.key && l.value == inc.value {
            found = true;
            break;
        }
    }
    if !found {
        return Err(SigniaError::invalid_argument("inclusion leaf not present in proof"));
    }

    // Start with leaf hash
    let payload = format!("{}={}", inc.key, inc.value);
    let mut h = crate::hash::hash_merkle_leaf_hex(proof.hash_alg.as_str(), payload.as_bytes())?;

    for s in &inc.siblings {
        let side = s.side.as_str();
        if side != "left" && side != "right" {
            return Err(SigniaError::invalid_argument("sibling.side must be left or right"));
        }

        let left;
        let right;

        if side == "left" {
            left = s.hash.as_str();
            right = h.as_str();
        } else {
            left = h.as_str();
            right = s.hash.as_str();
        }

        h = crate::hash::hash_merkle_node_hex(proof.hash_alg.as_str(), left, right)?;
    }

    if h != proof.root {
        return Err(SigniaError::invariant("inclusion proof root mismatch"));
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn verify_smoke() {
        // Minimal schema
        let schema = SchemaV1 {
            version: "v1".to_string(),
            kind: "repo".to_string(),
            meta: json!({
                "name":"demo",
                "createdAt":"1970-01-01T00:00:00Z",
                "source":{"type":"path","locator":"artifact:/demo"},
                "normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}
            }),
            entities: vec![],
            edges: vec![],
        };

        let mut manifest = ManifestV1::new(
            "demo",
            crate::model::v1::LimitsV1 {
                max_files: 1,
                max_bytes: 1,
                max_nodes: 1,
                max_edges: 1,
                timeout_ms: 1,
                network: "deny".to_string(),
            },
        );
        // Bind schema digest later

        let schema_hash = crate::hash::hash_schema_v1_hex(&schema).unwrap();
        let manifest_hash = crate::hash::hash_manifest_v1_hex(&manifest).unwrap();

        manifest.schemas.push(crate::model::v1::SchemaRefV1 {
            name: "repo".to_string(),
            digest: schema_hash.clone(),
        });

        // Proof
        let mut leaves = vec![
            LeafV1 {
                key: "digest:schemaHash".to_string(),
                value: schema_hash.clone(),
            },
            LeafV1 {
                key: "digest:manifestHash".to_string(),
                value: manifest_hash.clone(),
            },
        ];
        leaves.sort_by(|a, b| a.key.cmp(&b.key));

        let mut tree = crate::merkle::MerkleTree::new(crate::merkle::MerkleTreeOptions {
            hash_alg: "sha256".to_string(),
            domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
            domain_node: crate::domain::MERKLE_NODE.to_string(),
        });
        for leaf in &leaves {
            let payload = format!("{}={}", leaf.key, leaf.value);
            tree.push_leaf(payload.as_bytes()).unwrap();
        }
        let root = tree.root_hex().unwrap();

        let mut proof = ProofV1::new("sha256", root);
        proof.leaves = leaves;

        let bundle = VerifyBundle {
            schema,
            manifest,
            proof: Some(proof),
        };

        let rep = verify_bundle(bundle, VerifyOptions::default()).unwrap();
        assert!(rep.ok);
        assert!(!rep.has_errors());
    }
}
