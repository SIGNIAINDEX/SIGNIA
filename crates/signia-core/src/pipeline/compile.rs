//! High-level compile orchestration for SIGNIA core.
//!
//! This module provides deterministic compile orchestration that can be used by:
//! - CLI (after it loads inputs and runs plugins)
//! - API service (after it receives payloads / runs plugins)
//! - CI systems (after checkout / fetching data)
//!
//! Core crate intentionally does not perform filesystem or network I/O.
//! Instead, callers provide:
//! - an `IrGraph` created by plugins or other producers
//! - compilation parameters (kind, meta, limits, etc.)
//!
//! Output is a fully verifiable bundle:
//! - SchemaV1 (structure graph)
//! - ManifestV1 (inputs/outputs/limits/plugins)
//! - ProofV1 (Merkle root over deterministic leaves)
//!
//! Determinism contract:
//! - timestamps are injected (no system time reads)
//! - ordering is canonicalized (BTreeMap ordering or explicit sort)
//! - hashes use crate hashing utilities (domain-separated)
//! - no randomness, no env var dependence

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};
use crate::pipeline::{infer, stages, Pipeline, PipelineContext, PipelineData};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

#[cfg(feature = "canonical-json")]
use crate::model::ir::{DefaultIdStrategy, IdStrategy, IrGraph};

#[cfg(feature = "canonical-json")]
use crate::model::v1::{
    InputRefV1, LimitsV1, ManifestV1, OutputRefV1, PluginRefV1, ProofV1, SchemaV1,
};

/// Compile request.
///
/// This is the minimal deterministic input required by the core compile orchestrator.
/// Higher-level tooling may have richer request objects and map them into this struct.
#[derive(Debug, Clone)]
pub struct CompileRequest {
    /// Schema kind (repo, dataset, openapi, workflow, etc).
    pub kind: String,

    /// Schema meta (JSON).
    #[cfg(feature = "canonical-json")]
    pub meta: Value,

    /// Deterministic timestamp for manifest/schema.
    pub created_at: String,

    /// Labels to record into manifest.
    pub labels: BTreeMap<String, String>,

    /// Input specs to record into manifest.
    pub inputs: Vec<InputSpec>,

    /// Output specs to record into manifest.
    pub outputs: Vec<OutputSpec>,

    /// Plugin specs to record into manifest (execution may occur outside core).
    pub plugins: Vec<PluginSpec>,

    /// Compilation limits to record into manifest.
    pub limits: LimitsSpec,

    /// If true, run deterministic inference on IR before emission.
    pub run_inference: bool,

    /// If true, build proof leaves for schema and manifest and compute Merkle root.
    pub build_proof: bool,
}

/// Minimal input specification (recorded into ManifestV1).
#[derive(Debug, Clone)]
pub struct InputSpec {
    pub r#type: String,
    pub locator: String,
    pub digest: Option<String>,
}

/// Minimal output specification (recorded into ManifestV1).
#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub r#type: String,
    pub locator: String,
    pub expected_digest: Option<String>,
}

/// Minimal plugin specification (recorded into ManifestV1).
#[derive(Debug, Clone)]
pub struct PluginSpec {
    pub name: String,
    pub version: String,
    #[cfg(feature = "canonical-json")]
    pub config: Option<Value>,
}

/// Limits specification.
#[derive(Debug, Clone)]
pub struct LimitsSpec {
    pub max_files: u64,
    pub max_bytes: u64,
    pub max_nodes: u64,
    pub max_edges: u64,
    pub timeout_ms: u64,
    pub network: String,
}

impl Default for LimitsSpec {
    fn default() -> Self {
        Self {
            max_files: 50_000,
            max_bytes: 512 * 1024 * 1024,
            max_nodes: 2_000_000,
            max_edges: 4_000_000,
            timeout_ms: 60_000,
            network: "deny".to_string(),
        }
    }
}

/// Output bundle for compilation.
#[derive(Debug, Clone)]
pub struct CompileBundle {
    #[cfg(feature = "canonical-json")]
    pub schema: SchemaV1,
    #[cfg(feature = "canonical-json")]
    pub manifest: ManifestV1,
    #[cfg(feature = "canonical-json")]
    pub proof: Option<ProofV1>,
}

/// Stats for presentation.
#[derive(Debug, Clone, Default)]
pub struct CompileStats {
    pub entities: usize,
    pub edges: usize,
    pub leaf_count: usize,
}

/// A compile report includes bundle + diagnostics + stats.
#[derive(Debug, Clone)]
pub struct CompileReport {
    pub bundle: CompileBundle,
    pub diagnostics: Vec<crate::pipeline::PipelineDiagnostic>,
    pub stats: CompileStats,
}

#[cfg(feature = "canonical-json")]
impl CompileRequest {
    pub fn to_manifest_v1(&self, schema_digest_hex: Option<String>) -> ManifestV1 {
        let limits = LimitsV1 {
            max_files: self.limits.max_files,
            max_bytes: self.limits.max_bytes,
            max_nodes: self.limits.max_nodes,
            max_edges: self.limits.max_edges,
            timeout_ms: self.limits.timeout_ms,
            network: self.limits.network.clone(),
        };

        let mut m = ManifestV1::new("signia-compile", limits);

        // Record schemas by digest if provided (caller may compute).
        if let Some(d) = schema_digest_hex {
            m.add_schema(crate::model::v1::SchemaRefV1 {
                name: self.kind.clone(),
                digest: d,
            });
        }

        for i in &self.inputs {
            m.add_input(InputRefV1 {
                r#type: i.r#type.clone(),
                locator: i.locator.clone(),
                digest: i.digest.clone(),
            });
        }

        for o in &self.outputs {
            m.add_output(OutputRefV1 {
                r#type: o.r#type.clone(),
                locator: o.locator.clone(),
                expected_digest: o.expected_digest.clone(),
            });
        }

        for p in &self.plugins {
            m.add_plugin(PluginRefV1 {
                name: p.name.clone(),
                version: p.version.clone(),
                config: p.config.clone(),
            });
        }

        if !self.labels.is_empty() {
            m.labels = Some(self.labels.clone());
        }

        m
    }
}

/// Compile orchestration from IR graph.
///
/// You may optionally supply a custom id strategy. If not supplied, a default stable strategy is used.
#[cfg(feature = "canonical-json")]
pub fn compile_from_ir(
    mut ir: IrGraph,
    req: CompileRequest,
    id_strategy: Option<&dyn IdStrategy>,
) -> SigniaResult<CompileReport> {
    // Basic IR sanity
    ir.validate_basic()?;

    // Enforce limits early
    if (ir.nodes.len() as u64) > req.limits.max_nodes {
        return Err(SigniaError::invalid_argument(format!(
            "IR exceeds max_nodes ({} > {})",
            ir.nodes.len(),
            req.limits.max_nodes
        )));
    }
    if (ir.edges.len() as u64) > req.limits.max_edges {
        return Err(SigniaError::invalid_argument(format!(
            "IR exceeds max_edges ({} > {})",
            ir.edges.len(),
            req.limits.max_edges
        )));
    }

    // Optionally run deterministic inference
    if req.run_inference {
        let opts = infer::InferenceOptions::default();
        let rep = infer::infer_ir(&mut ir, &opts)?;
        let _ = rep;
    }

    // Build pipeline context
    let mut ctx = PipelineContext::default();
    ctx.clock.now_iso8601 = req.created_at.clone();
    ctx.set_param("schema.kind", req.kind.clone());
    ctx.set_json_param("schema.meta", req.meta.clone());

    // Construct pipeline to emit schema
    let mut p = Pipeline::new();
    p.push_stage(stages::ValidateIrStage::new("ir.validate"));
    p.push_stage(stages::NormalizeIrStage::new("ir.normalize"));
    p.push_stage(stages::EmitSchemaV1Stage::new("emit.schema_v1"));

    let report_schema = p.run(ctx.clone(), PipelineData::Ir(ir))?;
    let schema = match report_schema.output {
        PipelineData::SchemaV1(s) => s,
        other => {
            return Err(SigniaError::invariant(format!(
                "pipeline did not emit SchemaV1, got {other:?}"
            )))
        }
    };

    let mut diagnostics = report_schema.diagnostics;

    // Compute canonical digests for schema and manifest
    let schema_hash_hex = crate::hash::hash_schema_v1_hex(&schema)?;

    // Build manifest
    let manifest = req.to_manifest_v1(Some(schema_hash_hex.clone()));
    let manifest_hash_hex = crate::hash::hash_manifest_v1_hex(&manifest)?;

    // Build proof if requested
    let proof = if req.build_proof {
        let mut leaves: Vec<crate::model::v1::LeafV1> = Vec::new();
        leaves.push(crate::model::v1::LeafV1 {
            key: "digest:schemaHash".to_string(),
            value: schema_hash_hex.clone(),
        });
        leaves.push(crate::model::v1::LeafV1 {
            key: "digest:manifestHash".to_string(),
            value: manifest_hash_hex.clone(),
        });

        // Optional: include kind and createdAt for traceability (hashed as values)
        leaves.push(crate::model::v1::LeafV1 {
            key: "meta:kind".to_string(),
            value: crate::hash::hash_bytes_hex(req.kind.as_bytes())?,
        });
        leaves.push(crate::model::v1::LeafV1 {
            key: "meta:createdAt".to_string(),
            value: crate::hash::hash_bytes_hex(req.created_at.as_bytes())?,
        });

        // Deterministic ordering
        leaves.sort_by(|a, b| a.key.cmp(&b.key));

        let mut tree = crate::merkle::MerkleTree::new(crate::merkle::MerkleTreeOptions {
            hash_alg: "sha256".to_string(),
            domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
            domain_node: crate::domain::MERKLE_NODE.to_string(),
        });

        for leaf in &leaves {
            let payload = format!("{}={}", leaf.key, leaf.value);
            tree.push_leaf(payload.as_bytes())?;
        }

        let root = tree.root_hex()?;
        let mut p = ProofV1::new("sha256", root);
        p.leaves = leaves;

        Some(p)
    } else {
        None
    };

    // Aggregate stats
    let stats = CompileStats {
        entities: schema.entities.len(),
        edges: schema.edges.len(),
        leaf_count: proof.as_ref().map(|p| p.leaves.len()).unwrap_or(0),
    };

    Ok(CompileReport {
        bundle: CompileBundle {
            schema,
            manifest,
            proof,
        },
        diagnostics,
        stats,
    })
}

#[cfg(test)]
#[cfg(feature = "canonical-json")]
mod tests {
    use super::*;
    use crate::model::ir::{IrEdge, IrNode};
    use serde_json::json;

    #[test]
    fn compile_from_ir_emits_bundle() {
        let mut ir = IrGraph::new();
        ir.insert_node(IrNode {
            id: "n1".to_string(),
            key: "repo:root".to_string(),
            node_type: "repo".to_string(),
            name: "demo".to_string(),
            attrs: BTreeMap::new(),
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();
        ir.insert_node(IrNode {
            id: "n2".to_string(),
            key: "file:readme".to_string(),
            node_type: "file".to_string(),
            name: "README.md".to_string(),
            attrs: BTreeMap::new(),
            digests: vec![],
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();
        ir.insert_edge(IrEdge {
            id: "e1".to_string(),
            key: "contains:root:readme".to_string(),
            edge_type: "contains".to_string(),
            from: "n1".to_string(),
            to: "n2".to_string(),
            attrs: BTreeMap::new(),
            provenance: None,
            diagnostics: vec![],
        })
        .unwrap();

        let req = CompileRequest {
            kind: "repo".to_string(),
            meta: json!({
                "name":"demo",
                "createdAt":"1970-01-01T00:00:00Z",
                "source":{"type":"path","locator":"artifact:/demo"},
                "normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}
            }),
            created_at: "1970-01-01T00:00:00Z".to_string(),
            labels: BTreeMap::new(),
            inputs: vec![InputSpec {
                r#type: "path".to_string(),
                locator: "artifact:/demo".to_string(),
                digest: None,
            }],
            outputs: vec![OutputSpec {
                r#type: "schema".to_string(),
                locator: "artifact:/out/schema.json".to_string(),
                expected_digest: None,
            }],
            plugins: vec![PluginSpec {
                name: "repo".to_string(),
                version: "v1".to_string(),
                config: None,
            }],
            limits: LimitsSpec::default(),
            run_inference: true,
            build_proof: true,
        };

        let rep = compile_from_ir(ir, req, Some(&DefaultIdStrategy::default())).unwrap();
        assert_eq!(rep.bundle.schema.version, "v1");
        assert_eq!(rep.bundle.manifest.version, "v1");
        assert!(rep.bundle.proof.is_some());
        assert!(rep.stats.entities >= 2);
        assert!(rep.stats.leaf_count >= 2);
    }
}
