//! Link graph generation for built-in plugin specs.
//!
//! This module builds a deterministic, content-free "link graph" of how plugin
//! types relate to each other (supports, outputs, and common pipelines).
//!
//! It is intended for:
//! - docs visualization
//! - UI navigation (plugin -> supported types -> typical artifacts)
//! - compatibility analysis in hosts
//!
//! Notes:
//! - This is a conservative graph based on `PluginSpec` declarations.
//! - It does not require instantiating plugins or executing them.
//! - Output is deterministic and stable.

#![cfg(feature = "builtin")]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::spec::PluginSpec;

/// A graph node id (stable string).
pub type NodeId = String;

/// A graph edge kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Supports,
    SuggestsArtifact,
    Related,
}

/// Graph node type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Plugin,
    InputType,
    Artifact,
    Tag,
}

/// A graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

/// A graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub meta: BTreeMap<String, String>,
}

/// Deterministic link graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkGraph {
    pub nodes: BTreeMap<NodeId, LinkNode>,
    pub edges: BTreeSet<(NodeId, NodeId, EdgeKind)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_meta: Vec<LinkEdge>,
}

impl LinkGraph {
    pub fn add_node(&mut self, node: LinkNode) {
        self.nodes.entry(node.id.clone()).or_insert(node);
    }

    pub fn add_edge(&mut self, edge: LinkEdge) {
        self.edges
            .insert((edge.from.clone(), edge.to.clone(), edge.kind.clone()));
        self.edge_meta.push(edge);
        // Keep edge_meta deterministically ordered.
        self.edge_meta.sort_by(|a, b| {
            (a.from.clone(), a.to.clone(), a.kind.clone())
                .cmp(&(b.from.clone(), b.to.clone(), b.kind.clone()))
        });
    }
}

/// Build a link graph from plugin specs.
///
/// Rules:
/// - For each plugin spec:
///   - Create a Plugin node
///   - Create InputType nodes for each supported input type
///   - Add edges: Plugin --supports--> InputType
///   - Create Tag nodes for common metadata "category" or "tag" keys
///   - Suggest artifacts based on known supported types (conservative)
pub fn build_link_graph(specs: &[PluginSpec]) -> LinkGraph {
    let mut g = LinkGraph::default();

    // Predefined artifact suggestions per input type (conservative).
    let mut suggestions: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    suggestions.insert("repo", vec!["schema", "manifest", "proof", "ir"]);
    suggestions.insert("dataset", vec!["schema", "manifest", "proof", "fingerprint", "ir"]);
    suggestions.insert("openapi", vec!["schema", "manifest", "proof", "ir"]);
    suggestions.insert("workflow", vec!["schema", "manifest", "proof", "ir"]);

    for spec in specs {
        let plugin_id = format!("plugin:{}", spec.id);
        g.add_node(LinkNode {
            id: plugin_id.clone(),
            kind: NodeKind::Plugin,
            label: spec.title.clone(),
            meta: {
                let mut m = BTreeMap::new();
                m.insert("id".to_string(), spec.id.clone());
                m.insert("version".to_string(), spec.version.clone());
                if !spec.description.is_empty() {
                    m.insert("description".to_string(), spec.description.clone());
                }
                m
            },
        });

        // Tags from metadata
        for key in ["category", "tag"] {
            if let Some(v) = spec.meta.get(key) {
                let tag_id = format!("tag:{}:{}", key, v);
                g.add_node(LinkNode {
                    id: tag_id.clone(),
                    kind: NodeKind::Tag,
                    label: v.clone(),
                    meta: {
                        let mut m = BTreeMap::new();
                        m.insert("key".to_string(), key.to_string());
                        m
                    },
                });
                g.add_edge(LinkEdge {
                    from: plugin_id.clone(),
                    to: tag_id,
                    kind: EdgeKind::Related,
                    meta: BTreeMap::new(),
                });
            }
        }

        // Supported types
        for t in &spec.supports {
            let t_id = format!("type:{t}");
            g.add_node(LinkNode {
                id: t_id.clone(),
                kind: NodeKind::InputType,
                label: t.clone(),
                meta: BTreeMap::new(),
            });
            g.add_edge(LinkEdge {
                from: plugin_id.clone(),
                to: t_id.clone(),
                kind: EdgeKind::Supports,
                meta: BTreeMap::new(),
            });

            // Suggest artifacts for this type.
            if let Some(arts) = suggestions.get(t.as_str()) {
                for a in arts {
                    let a_id = format!("artifact:{a}");
                    g.add_node(LinkNode {
                        id: a_id.clone(),
                        kind: NodeKind::Artifact,
                        label: a.to_string(),
                        meta: BTreeMap::new(),
                    });
                    g.add_edge(LinkEdge {
                        from: t_id.clone(),
                        to: a_id,
                        kind: EdgeKind::SuggestsArtifact,
                        meta: {
                            let mut m = BTreeMap::new();
                            m.insert("confidence".to_string(), "conservative".to_string());
                            m
                        },
                    });
                }
            }
        }
    }

    g
}

/// Convert the link graph to a JSON value for API output.
pub fn link_graph_to_json(g: &LinkGraph) -> serde_json::Value {
    let nodes = g
        .nodes
        .values()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "kind": format!("{:?}", n.kind).to_ascii_lowercase(),
                "label": n.label,
                "meta": n.meta,
            })
        })
        .collect::<Vec<_>>();

    let edges = g
        .edge_meta
        .iter()
        .map(|e| {
            serde_json::json!({
                "from": e.from,
                "to": e.to,
                "kind": format!("{:?}", e.kind).to_ascii_lowercase(),
                "meta": e.meta,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "nodes": nodes,
        "edges": edges,
        "counts": {
            "nodes": g.nodes.len(),
            "edges": g.edge_meta.len(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::PluginSpec;

    #[test]
    fn graph_has_support_edges() {
        let specs = vec![
            PluginSpec::new("builtin.repo", "Repo", "0.1.0").support("repo"),
            PluginSpec::new("builtin.dataset", "Dataset", "0.1.0").support("dataset"),
        ];
        let g = build_link_graph(&specs);
        assert!(g.nodes.contains_key("plugin:builtin.repo"));
        assert!(g.nodes.contains_key("type:repo"));
        // Edge existence is implied by edge_meta.
        assert!(g.edge_meta.iter().any(|e| e.kind == EdgeKind::Supports));
    }

    #[test]
    fn json_output_has_counts() {
        let specs = vec![PluginSpec::new("x", "X", "0.1.0").support("repo")];
        let g = build_link_graph(&specs);
        let j = link_graph_to_json(&g);
        assert!(j.get("counts").is_some());
    }
}
