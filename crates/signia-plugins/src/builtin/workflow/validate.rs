//! Validation logic for the built-in `workflow` plugin.
//!
//! This module performs strict, deterministic validation of workflow inputs
//! before they are compiled into IR.
//!
//! Guarantees:
//! - No filesystem or network I/O
//! - No randomness or time-based behavior
//! - Stable error messages for identical inputs
//!
//! This validation is intended to be reusable by:
//! - CLI preflight checks
//! - API request validation
//! - CI pipelines

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde_json::Value;

/// Validate a workflow JSON object.
///
/// Required top-level fields:
/// - name: string
/// - nodes: array
///
/// Optional:
/// - version: string
/// - edges: array
///
/// This function does not mutate the input.
pub fn validate_workflow(v: &Value) -> Result<()> {
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("workflow must be a JSON object"))?;

    let name = obj
        .get("name")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("workflow.name is required"))?;
    if name.trim().is_empty() {
        return Err(anyhow!("workflow.name must not be empty"));
    }

    let nodes = obj
        .get("nodes")
        .and_then(|x| x.as_array())
        .ok_or_else(|| anyhow!("workflow.nodes must be an array"))?;
    if nodes.is_empty() {
        return Err(anyhow!("workflow.nodes must not be empty"));
    }

    let edges = obj
        .get("edges")
        .and_then(|x| x.as_array())
        .unwrap_or(&Vec::new());

    validate_nodes(nodes)?;
    validate_edges(nodes, edges)?;

    Ok(())
}

fn validate_nodes(nodes: &[Value]) -> Result<()> {
    let mut ids = BTreeSet::<String>::new();

    for (idx, n) in nodes.iter().enumerate() {
        let obj = n
            .as_object()
            .ok_or_else(|| anyhow!("workflow.nodes[{idx}] must be an object"))?;

        let id = obj
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("workflow.nodes[{idx}].id is required"))?;
        if id.trim().is_empty() {
            return Err(anyhow!("workflow.nodes[{idx}].id must not be empty"));
        }
        if !ids.insert(id.to_string()) {
            return Err(anyhow!("duplicate workflow node id: {id}"));
        }

        let ty = obj
            .get("type")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("workflow.nodes[{idx}].type is required"))?;
        if ty.trim().is_empty() {
            return Err(anyhow!("workflow.nodes[{idx}].type must not be empty"));
        }

        if let Some(meta) = obj.get("meta") {
            if !meta.is_object() {
                return Err(anyhow!(
                    "workflow.nodes[{idx}].meta must be an object if present"
                ));
            }
        }

        if let Some(inputs) = obj.get("inputs") {
            if !inputs.is_object() {
                return Err(anyhow!(
                    "workflow.nodes[{idx}].inputs must be an object if present"
                ));
            }
        }
    }

    Ok(())
}

fn validate_edges(nodes: &[Value], edges: &[Value]) -> Result<()> {
    let mut node_ids = BTreeSet::<String>::new();
    for n in nodes {
        let id = n
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("node missing id during edge validation"))?;
        node_ids.insert(id.to_string());
    }

    for (idx, e) in edges.iter().enumerate() {
        let obj = e
            .as_object()
            .ok_or_else(|| anyhow!("workflow.edges[{idx}] must be an object"))?;

        let from = obj
            .get("from")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("workflow.edges[{idx}].from is required"))?;
        let to = obj
            .get("to")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("workflow.edges[{idx}].to is required"))?;

        if !node_ids.contains(from) {
            return Err(anyhow!(
                "workflow.edges[{idx}].from references unknown node: {from}"
            ));
        }
        if !node_ids.contains(to) {
            return Err(anyhow!(
                "workflow.edges[{idx}].to references unknown node: {to}"
            ));
        }

        let kind = obj
            .get("kind")
            .and_then(|x| x.as_str())
            .ok_or_else(|| anyhow!("workflow.edges[{idx}].kind is required"))?;
        if !matches!(kind, "data" | "control" | "event") {
            return Err(anyhow!(
                "workflow.edges[{idx}].kind must be one of data|control|event"
            ));
        }

        if let Some(label) = obj.get("label") {
            if !label.is_string() {
                return Err(anyhow!(
                    "workflow.edges[{idx}].label must be a string if present"
                ));
            }
        }
    }

    Ok(())
}

/// Build a deterministic summary useful for debugging and CI logs.
pub fn workflow_summary(v: &Value) -> Result<BTreeMap<String, usize>> {
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("workflow must be an object"))?;

    let nodes = obj
        .get("nodes")
        .and_then(|x| x.as_array())
        .ok_or_else(|| anyhow!("workflow.nodes missing"))?;

    let edges = obj
        .get("edges")
        .and_then(|x| x.as_array())
        .unwrap_or(&Vec::new());

    let mut out = BTreeMap::new();
    out.insert("nodes".to_string(), nodes.len());
    out.insert("edges".to_string(), edges.len());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_workflow_passes() {
        let v = json!({
            "name": "demo",
            "nodes": [
                { "id": "a", "type": "http" },
                { "id": "b", "type": "llm" }
            ],
            "edges": [
                { "from": "a", "to": "b", "kind": "data" }
            ]
        });
        validate_workflow(&v).unwrap();
    }

    #[test]
    fn duplicate_node_fails() {
        let v = json!({
            "name": "demo",
            "nodes": [
                { "id": "a", "type": "x" },
                { "id": "a", "type": "y" }
            ]
        });
        assert!(validate_workflow(&v).is_err());
    }

    #[test]
    fn bad_edge_kind_fails() {
        let v = json!({
            "name": "demo",
            "nodes": [
                { "id": "a", "type": "x" },
                { "id": "b", "type": "y" }
            ],
            "edges": [
                { "from": "a", "to": "b", "kind": "unknown" }
            ]
        });
        assert!(validate_workflow(&v).is_err());
    }
}
