//! Built-in `repo` plugin for SIGNIA.
//!
//! This plugin handles Git repository–like inputs that have already been
//! materialized by the host into a deterministic structure.
//!
//! Responsibilities:
//! - validate repo metadata
//! - convert repo structure into canonical IR
//! - emit schema / manifest artifacts via pipeline context
//!
//! Non-responsibilities:
//! - cloning repositories
//! - network access
//! - filesystem access
//!
//! All inputs must be provided by the host in structured form.

#![cfg(feature = "builtin")]

use anyhow::Result;

use signia_core::model::ir::{IrEdge, IrGraph, IrNode};
use signia_core::pipeline::context::PipelineContext;

use crate::plugin::{Plugin, PluginInput, PluginOutput};
use crate::registry::PluginRegistry;
use crate::spec::PluginSpec;

/// Register the repo plugin.
pub fn register(registry: &mut PluginRegistry) {
    let spec = PluginSpec::new("builtin.repo", "Repository Plugin", "0.1.0")
        .support("repo")
        .limit("max_nodes", 200_000)
        .limit("max_edges", 400_000)
        .want("network", false)
        .want("filesystem", false)
        .meta("category", "source");

    registry
        .register(spec, Box::new(RepoPlugin))
        .expect("failed to register builtin.repo");
}

/// Repo plugin implementation.
pub struct RepoPlugin;

impl Plugin for RepoPlugin {
    fn name(&self) -> &str {
        "repo"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn supports(&self, input_type: &str) -> bool {
        input_type == "repo"
    }

    fn execute(&self, input: &PluginInput) -> Result<PluginOutput> {
        let ctx = match input {
            PluginInput::Pipeline(ctx) => ctx,
            _ => anyhow::bail!("repo plugin requires pipeline input"),
        };

        execute_repo(ctx)?;
        Ok(PluginOutput::None)
    }
}

/// Core execution logic for repo plugin.
fn execute_repo(ctx: &mut PipelineContext) -> Result<()> {
    // Expect repo metadata to be present in pipeline inputs.
    let meta = ctx
        .inputs
        .get("repo")
        .ok_or_else(|| anyhow::anyhow!("missing repo input"))?;

    let repo_name = meta
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("repo.name missing or invalid"))?;

    let mut graph = IrGraph::new();

    // Root node
    let root = IrNode::new("repo", repo_name);
    let root_id = graph.add_node(root);

    // Files
    if let Some(files) = meta.get("files").and_then(|v| v.as_array()) {
        for file in files {
            let path = file
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("file.path missing"))?;

            let node = IrNode::new("file", path);
            let node_id = graph.add_node(node);

            graph.add_edge(IrEdge::new(root_id, node_id, "contains"));
        }
    }

    ctx.ir = Some(graph);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use signia_core::pipeline::context::PipelineConfig;

    #[test]
    fn repo_plugin_executes() {
        let mut ctx = PipelineContext::new(PipelineConfig::default());
        ctx.inputs.insert(
            "repo".to_string(),
            json!({
                "name": "test-repo",
                "files": [
                    { "path": "src/lib.rs" },
                    { "path": "README.md" }
                ]
            }),
        );

        let plugin = RepoPlugin;
        let out = plugin.execute(&PluginInput::Pipeline(&mut ctx)).unwrap();
        matches!(out, PluginOutput::None);

        assert!(ctx.ir.is_some());
        let graph = ctx.ir.unwrap();
        assert_eq!(graph.nodes.len(), 3);
    }
}
