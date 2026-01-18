//! Built-in pipeline stages for SIGNIA.
//!
//! These stages are generic, deterministic building blocks used by higher-level
//! tooling (CLI/API). They avoid any filesystem/network I/O.
//!
//! Included stages:
//! - JSON validation helpers (structural)
//! - IR normalization helpers (ordering, id assignment hooks)
//! - Emission helpers (IR -> SchemaV1)
//! - Proof construction helper glue (leaves -> Merkle root)
//!
//! Note: complex plugin execution lives in `signia-plugins` crate (not core).
//! Core stages are intentionally minimal.

use std::collections::{BTreeMap, BTreeSet};

use crate::errors::{SigniaError, SigniaResult};
use crate::pipeline::{PipelineContext, PipelineData, Stage};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

#[cfg(feature = "canonical-json")]
use crate::model::ir::{DefaultIdStrategy, IdStrategy, IrGraph};

#[cfg(feature = "canonical-json")]
use crate::model::v1::{ProofV1, SchemaV1};

/// Stage: Validate that a `PipelineData::Json` value is an object.
///
/// Useful for quick sanity checks before plugin code uses `.as_object().unwrap()`.
pub struct ValidateJsonObjectStage {
    id: String,
}

impl ValidateJsonObjectStage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Stage for ValidateJsonObjectStage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for ValidateJsonObjectStage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            match input {
                PipelineData::Json(v) => {
                    if !v.is_object() {
                        ctx.push_error("json.not_object", "expected JSON object");
                        return Err(SigniaError::invalid_argument("expected JSON object"));
                    }
                    Ok(PipelineData::Json(v))
                }
                other => Err(SigniaError::invalid_argument(format!(
                    "expected PipelineData::Json, got {other:?}"
                ))),
            }
        }
    }
}

/// Stage: Validate basic IR invariants.
pub struct ValidateIrStage {
    id: String,
}

impl ValidateIrStage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Stage for ValidateIrStage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for ValidateIrStage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            match input {
                PipelineData::Ir(g) => {
                    g.validate_basic()?;
                    ctx.push_info("ir.validated", "IR basic validation succeeded");
                    Ok(PipelineData::Ir(g))
                }
                other => Err(SigniaError::invalid_argument(format!(
                    "expected PipelineData::Ir, got {other:?}"
                ))),
            }
        }
    }
}

/// Stage: Normalize IR ordering (no-op for IR maps, but can enforce stable ordering of internal collections).
///
/// For the current IR design using BTreeMap/BTreeSet, ordering is already stable.
/// This stage is still useful as an explicit step and future hook.
pub struct NormalizeIrStage {
    id: String,
}

impl NormalizeIrStage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Stage for NormalizeIrStage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for NormalizeIrStage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            match input {
                PipelineData::Ir(mut g) => {
                    // This is a hook for future normalization. For now we ensure basic validity and
                    // emit a stable summary.
                    g.validate_basic()?;

                    let node_count = g.nodes.len();
                    let edge_count = g.edges.len();

                    ctx.push_info(
                        "ir.normalized",
                        format!("IR normalized (nodes={node_count}, edges={edge_count})"),
                    );

                    Ok(PipelineData::Ir(g))
                }
                other => Err(SigniaError::invalid_argument(format!(
                    "expected PipelineData::Ir, got {other:?}"
                ))),
            }
        }
    }
}

/// Stage: Emit SchemaV1 from IR.
///
/// Inputs:
/// - PipelineData::Ir
/// Requires ctx params:
/// - `schema.kind`
/// - `schema.meta` (JSON string) OR ctx.json_params["schema.meta"] if enabled
///
/// Output:
/// - PipelineData::SchemaV1
pub struct EmitSchemaV1Stage {
    id: String,
}

impl EmitSchemaV1Stage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    #[cfg(feature = "canonical-json")]
    fn meta_from_ctx(ctx: &PipelineContext) -> SigniaResult<Value> {
        // Prefer json_params if present
        if let Some(v) = ctx.get_json_param("schema.meta") {
            return Ok(v.clone());
        }

        // Fallback to string param
        if let Some(s) = ctx.get_param("schema.meta") {
            let v: Value = serde_json::from_str(s)
                .map_err(|e| SigniaError::serialization(format!("failed to parse schema.meta JSON: {e}")))?;
            return Ok(v);
        }

        Err(SigniaError::invalid_argument(
            "missing schema.meta (set ctx.json_params[\"schema.meta\"] or ctx.params[\"schema.meta\"])",
        ))
    }
}

impl Stage for EmitSchemaV1Stage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for EmitSchemaV1Stage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            let kind = ctx
                .get_param("schema.kind")
                .ok_or_else(|| SigniaError::invalid_argument("missing schema.kind in ctx params"))?
                .to_string();

            let meta = Self::meta_from_ctx(ctx)?;

            match input {
                PipelineData::Ir(g) => {
                    g.validate_basic()?;

                    // Default deterministic id strategy; higher layers may override by implementing IdStrategy.
                    let ids = DefaultIdStrategy::default();
                    let schema = g.emit_schema_v1(&kind, meta, &ids)?;

                    ctx.push_info("emit.schema_v1", "emitted SchemaV1 from IR");

                    Ok(PipelineData::SchemaV1(schema))
                }
                other => Err(SigniaError::invalid_argument(format!(
                    "expected PipelineData::Ir, got {other:?}"
                ))),
            }
        }
    }
}

/// Stage: Build a proof Merkle root from given leaf entries.
///
/// Inputs:
/// - PipelineData::Json containing {"hashAlg":"sha256","leaves":[{"key":"...","value":"..."}, ...]}
///
/// Output:
/// - PipelineData::ProofV1
///
/// The actual Merkle computation is implemented in `crate::merkle`.
pub struct BuildProofV1Stage {
    id: String,
}

impl BuildProofV1Stage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Stage for BuildProofV1Stage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for BuildProofV1Stage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            let v = match input {
                PipelineData::Json(v) => v,
                other => {
                    return Err(SigniaError::invalid_argument(format!(
                        "expected PipelineData::Json, got {other:?}"
                    )))
                }
            };

            let obj = v.as_object().ok_or_else(|| SigniaError::invalid_argument("proof input must be an object"))?;
            let hash_alg = obj
                .get("hashAlg")
                .and_then(|x| x.as_str())
                .ok_or_else(|| SigniaError::invalid_argument("proof input missing hashAlg string"))?
                .to_string();

            let leaves_val = obj
                .get("leaves")
                .and_then(|x| x.as_array())
                .ok_or_else(|| SigniaError::invalid_argument("proof input missing leaves array"))?;

            let mut leaves: Vec<crate::model::v1::LeafV1> = Vec::new();
            for lv in leaves_val {
                let o = lv.as_object().ok_or_else(|| SigniaError::invalid_argument("leaf must be an object"))?;
                let key = o
                    .get("key")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| SigniaError::invalid_argument("leaf.key must be a string"))?;
                let value = o
                    .get("value")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| SigniaError::invalid_argument("leaf.value must be a string"))?;
                leaves.push(crate::model::v1::LeafV1 {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }

            // Deterministic ordering of leaves by key.
            leaves.sort_by(|a, b| a.key.cmp(&b.key));

            // Build Merkle root from leaf hashes using the core merkle utilities.
            let mut tree = crate::merkle::MerkleTree::new(crate::merkle::MerkleTreeOptions {
                hash_alg: hash_alg.clone(),
                domain_leaf: crate::domain::MERKLE_LEAF.to_string(),
                domain_node: crate::domain::MERKLE_NODE.to_string(),
            });

            for leaf in &leaves {
                let payload = format!("{}={}", leaf.key, leaf.value);
                tree.push_leaf(payload.as_bytes())?;
            }

            let root = tree.root_hex()?;

            let mut proof = ProofV1::new(hash_alg, root);
            proof.leaves = leaves;

            ctx.push_info("proof.built", "built ProofV1 Merkle root");

            Ok(PipelineData::ProofV1(proof))
        }
    }
}

/// Stage: Extract a list of unique entity types from a SchemaV1 into JSON.
///
/// Inputs:
/// - PipelineData::SchemaV1
///
/// Output:
/// - PipelineData::Json: {"entityTypes":[...],"edgeTypes":[...],"entities":N,"edges":M}
pub struct SchemaSummaryStage {
    id: String,
}

impl SchemaSummaryStage {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Stage for SchemaSummaryStage {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
        #[cfg(not(feature = "canonical-json"))]
        {
            let _ = ctx;
            let _ = input;
            return Err(SigniaError::invalid_argument(
                "canonical-json feature is required for SchemaSummaryStage",
            ));
        }

        #[cfg(feature = "canonical-json")]
        {
            let schema = match input {
                PipelineData::SchemaV1(s) => s,
                other => {
                    return Err(SigniaError::invalid_argument(format!(
                        "expected PipelineData::SchemaV1, got {other:?}"
                    )))
                }
            };

            let mut entity_types: BTreeSet<String> = BTreeSet::new();
            for e in &schema.entities {
                entity_types.insert(e.r#type.clone());
            }

            let mut edge_types: BTreeSet<String> = BTreeSet::new();
            for ed in &schema.edges {
                edge_types.insert(ed.r#type.clone());
            }

            let out = serde_json::json!({
                "entities": schema.entities.len(),
                "edges": schema.edges.len(),
                "entityTypes": entity_types.into_iter().collect::<Vec<_>>(),
                "edgeTypes": edge_types.into_iter().collect::<Vec<_>>(),
            });

            ctx.push_info("schema.summary", "created schema summary");

            Ok(PipelineData::Json(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{Pipeline, PipelineContext};

    #[test]
    #[cfg(feature = "canonical-json")]
    fn stages_emit_schema_and_summary() {
        // Minimal IR with 2 nodes and 1 edge
        let mut g = IrGraph::new();
        g.insert_node(crate::model::ir::IrNode {
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

        g.insert_node(crate::model::ir::IrNode {
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

        g.insert_edge(crate::model::ir::IrEdge {
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

        let mut ctx = PipelineContext::default();
        ctx.set_param("schema.kind", "repo");
        ctx.set_json_param(
            "schema.meta",
            serde_json::json!({
                "name":"demo",
                "createdAt":"1970-01-01T00:00:00Z",
                "source":{"type":"path","locator":"artifact:/demo"},
                "normalization":{"policyVersion":"v1","pathRoot":"artifact:/","newline":"lf","encoding":"utf-8","symlinks":"deny","network":"deny"}
            }),
        );

        let mut p = Pipeline::new();
        p.push_stage(ValidateIrStage::new("ir.validate"));
        p.push_stage(NormalizeIrStage::new("ir.normalize"));
        p.push_stage(EmitSchemaV1Stage::new("emit.schema_v1"));
        p.push_stage(SchemaSummaryStage::new("schema.summary"));

        let report = p.run(ctx, PipelineData::Ir(g)).unwrap();
        match report.output {
            PipelineData::Json(v) => {
                assert_eq!(v["entities"], 2);
                assert_eq!(v["edges"], 1);
            }
            _ => panic!("expected json output"),
        }
    }
}
