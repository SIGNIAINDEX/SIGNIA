//! Built-in `workflow` plugin for SIGNIA.
//!
//! This plugin models a "workflow" as a deterministic structure that can be
//! reasoned about on-chain.
//!
//! Input expectations (provided by host):
//! - JSON object under `ctx.inputs["workflow"]`
//! - schema:
//!   {
//!     "name": "string",
//!     "version": "string (optional)",
//!     "nodes": [
//!       { "id": "string", "type": "string", "inputs": {...}, "meta": {...} }
//!     ],
//!     "edges": [
//!       { "from": "nodeId", "to": "nodeId", "kind": "data|control|event", "label": "string (optional)" }
//!     ]
//!   }
//!
//! Responsibilities:
//! - validate and normalize workflow graph
//! - enforce determinism (stable ordering, stable ids)
//! - build `IrGraph` and attach stable fingerprint
//!
//! Non-responsibilities:
//! - executing the workflow
//! - contacting external services
//! - reading files

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use serde_json::Value;

use signia_core::determinism::hashing::hash_bytes_hex;
use signia_core::model::ir::{IrEdge, IrGraph, IrNode};
use signia_core::pipeline::context::PipelineContext;

use crate::plugin::{Plugin, PluginInput, PluginOutput};
use crate::registry::PluginRegistry;
use crate::spec::PluginSpec;

/// Register the workflow plugin.
pub fn register(registry: &mut PluginRegistry) {
    let spec = PluginSpec::new("builtin.workflow", "Workflow Plugin", "0.1.0")
        .support("workflow")
        .limit("max_nodes", 200_000)
        .limit("max_edges", 400_000)
        .want("network", false)
        .want("filesystem", false)
        .meta("category", "orchestration");

    registry
        .register(spec, Box::new(WorkflowPlugin))
        .expect("failed to register builtin.workflow");
}

/// Workflow plugin implementation.
pub struct WorkflowPlugin;

impl Plugin for WorkflowPlugin {
    fn name(&self) -> &str {
        "workflow"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn supports(&self, input_type: &str) -> bool {
        input_type == "workflow"
    }

    fn execute(&self, input: &PluginInput) -> Result<PluginOutput> {
        let ctx = match input {
            PluginInput::Pipeline(ctx) => ctx,
            _ => anyhow::bail!("workflow plugin requires pipeline input"),
        };

        execute_workflow(ctx)?;
        Ok(PluginOutput::None)
    }
}

fn execute_workflow(ctx: &mut PipelineContext) -> Result<()> {
    let v = ctx
        .inputs
        .get("workflow")
        .ok_or_else(|| anyhow!("missing workflow input"))?;

    let name = get_str(v, "name")?;
    let version = v.get("version").and_then(|x| x.as_str()).unwrap_or("unknown");

    let nodes = v
        .get("nodes")
        .and_then(|x| x.as_array())
        .ok_or_else(|| anyhow!("workflow.nodes missing or invalid"))?;

    let edges = v
        .get("edges")
        .and_then(|x| x.as_array())
        .ok_or_else(|| anyhow!("workflow.edges missing or invalid"))?;

    // Validate nodes: unique ids
    let mut node_ids = BTreeSet::<String>::new();
    for n in nodes {
        let id = get_str(n, "id")?;
        if !node_ids.insert(id.to_string()) {
            return Err(anyhow!("duplicate workflow node id: {id}"));
        }
        // Require type
        let _t = get_str(n, "type")?;
    }

    // Validate edges reference existing nodes
    for e in edges {
        let from = get_str(e, "from")?;
        let to = get_str(e, "to")?;
        if !node_ids.contains(from) {
            return Err(anyhow!("edge.from references unknown node: {from}"));
        }
        if !node_ids.contains(to) {
            return Err(anyhow!("edge.to references unknown node: {to}"));
        }
        let kind = get_str(e, "kind")?;
        if !matches!(kind, "data" | "control" | "event") {
            return Err(anyhow!("invalid edge kind: {kind}"));
        }
    }

    // Deterministic ordering:
    // - nodes sorted by id
    // - edges sorted by (from,to,kind,label)
    let mut nodes_sorted: Vec<&Value> = nodes.iter().collect();
    nodes_sorted.sort_by(|a, b| get_str(a, "id").unwrap().cmp(get_str(b, "id").unwrap()));

    let mut edges_sorted: Vec<&Value> = edges.iter().collect();
    edges_sorted.sort_by(|a, b| {
        let ak = (
            get_str(a, "from").unwrap(),
            get_str(a, "to").unwrap(),
            get_str(a, "kind").unwrap(),
            a.get("label").and_then(|x| x.as_str()).unwrap_or(""),
        );
        let bk = (
            get_str(b, "from").unwrap(),
            get_str(b, "to").unwrap(),
            get_str(b, "kind").unwrap(),
            b.get("label").and_then(|x| x.as_str()).unwrap_or(""),
        );
        ak.cmp(&bk)
    });

    // Build IR
    let mut graph = IrGraph::new();
    let root_id = graph.add_node(IrNode::new("workflow", name));
    let ver_id = graph.add_node(IrNode::new("version", version));
    graph.add_edge(IrEdge::new(root_id, ver_id, "version"));

    let mut id_to_ir: BTreeMap<String, u64> = BTreeMap::new();

    for n in nodes_sorted {
        let id = get_str(n, "id")?;
        let t = get_str(n, "type")?;
        let label = format!("{id}:{t}");
        let nid = graph.add_node(IrNode::new("node", label));

        // Attach node type as a scalar node
        let tid = graph.add_node(IrNode::new("type", t));
        graph.add_edge(IrEdge::new(nid, tid, "has"));

        // Attach stable metadata keys (if provided)
        if let Some(meta) = n.get("meta").and_then(|x| x.as_object()) {
            let mut keys: Vec<&String> = meta.keys().collect();
            keys.sort();
            for k in keys {
                let val = meta.get(k).unwrap();
                let vs = if val.is_string() {
                    val.as_str().unwrap().to_string()
                } else {
                    // stable JSON string for non-string values
                    serde_json::to_string(val)?
                };
                let mid = graph.add_node(IrNode::new("meta", format!("{k}={vs}")));
                graph.add_edge(IrEdge::new(nid, mid, "meta"));
            }
        }

        graph.add_edge(IrEdge::new(root_id, nid, "contains"));
        id_to_ir.insert(id.to_string(), nid);
    }

    for e in edges_sorted {
        let from = get_str(e, "from")?;
        let to = get_str(e, "to")?;
        let kind = get_str(e, "kind")?;
        let label = e.get("label").and_then(|x| x.as_str()).unwrap_or("");

        let from_id = *id_to_ir.get(from).unwrap();
        let to_id = *id_to_ir.get(to).unwrap();

        // Represent as an edge node for richer modeling
        let en = graph.add_node(IrNode::new("edge", format!("{from}->{to}:{kind}:{label}")));
        graph.add_edge(IrEdge::new(root_id, en, "contains"));
        graph.add_edge(IrEdge::new(en, from_id, "from"));
        graph.add_edge(IrEdge::new(en, to_id, "to"));

        let k_id = graph.add_node(IrNode::new("kind", kind));
        graph.add_edge(IrEdge::new(en, k_id, "has"));
    }

    // Fingerprint: stable text concatenation (nodes + edges)
    let fingerprint = workflow_fingerprint(name, version, &nodes_sorted, &edges_sorted)?;
    ctx.metadata
        .insert("workflowFingerprint".to_string(), Value::String(fingerprint));

    ctx.ir = Some(graph);
    Ok(())
}

fn workflow_fingerprint(
    name: &str,
    version: &str,
    nodes_sorted: &[&Value],
    edges_sorted: &[&Value],
) -> Result<String> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"workflow\n");
    buf.extend_from_slice(name.as_bytes());
    buf.extend_from_slice(b"\n");
    buf.extend_from_slice(version.as_bytes());
    buf.extend_from_slice(b"\n");

    buf.extend_from_slice(b"nodes\n");
    for n in nodes_sorted {
        let id = get_str(n, "id")?;
        let t = get_str(n, "type")?;
        buf.extend_from_slice(id.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(t.as_bytes());
        buf.extend_from_slice(b"\n");

        // Meta keys stable
        if let Some(meta) = n.get("meta").and_then(|x| x.as_object()) {
            let mut keys: Vec<&String> = meta.keys().collect();
            keys.sort();
            for k in keys {
                let val = meta.get(k).unwrap();
                let vs = if val.is_string() {
                    val.as_str().unwrap().to_string()
                } else {
                    serde_json::to_string(val)?
                };
                buf.extend_from_slice(b"meta\t");
                buf.extend_from_slice(k.as_bytes());
                buf.extend_from_slice(b"=");
                buf.extend_from_slice(vs.as_bytes());
                buf.extend_from_slice(b"\n");
            }
        }
    }

    buf.extend_from_slice(b"edges\n");
    for e in edges_sorted {
        let from = get_str(e, "from")?;
        let to = get_str(e, "to")?;
        let kind = get_str(e, "kind")?;
        let label = e.get("label").and_then(|x| x.as_str()).unwrap_or("");

        buf.extend_from_slice(from.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(to.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(kind.as_bytes());
        buf.extend_from_slice(b"\t");
        buf.extend_from_slice(label.as_bytes());
        buf.extend_from_slice(b"\n");
    }

    hash_bytes_hex(&buf)
}

fn get_str<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("missing or invalid string field: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use signia_core::pipeline::context::PipelineConfig;

    #[test]
    fn workflow_plugin_executes_and_fingerprints() {
        let mut ctx = PipelineContext::new(PipelineConfig::default());
        ctx.inputs.insert(
            "workflow".to_string(),
            json!({
                "name": "demo",
                "version": "v1",
                "nodes": [
                    {"id":"a","type":"http","meta":{"url":"https://example.com"}},
                    {"id":"b","type":"llm","meta":{"model":"gpt"}}
                ],
                "edges": [
                    {"from":"a","to":"b","kind":"data","label":"response"}
                ]
            }),
        );

        let plugin = WorkflowPlugin;
        plugin.execute(&PluginInput::Pipeline(&mut ctx)).unwrap();

        assert!(ctx.ir.is_some());
        assert!(ctx.metadata.get("workflowFingerprint").is_some());
    }

    #[test]
    fn duplicate_node_id_fails() {
        let mut ctx = PipelineContext::new(PipelineConfig::default());
        ctx.inputs.insert(
            "workflow".to_string(),
            json!({
                "name": "demo",
                "nodes": [
                    {"id":"a","type":"x"},
                    {"id":"a","type":"y"}
                ],
                "edges": []
            }),
        );

        let plugin = WorkflowPlugin;
        let r = plugin.execute(&PluginInput::Pipeline(&mut ctx));
        assert!(r.is_err());
    }
}
