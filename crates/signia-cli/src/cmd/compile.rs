use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::io::{export, input};
use crate::output;

#[derive(Debug, Serialize)]
pub struct CompileOut {
    pub kind: String,
    pub schema_id: String,
    pub manifest_id: String,
    pub proof_id: String,
    pub out_dir: String,
    pub metadata: BTreeMap<String, String>,
}

pub async fn run(store_root: &str, input_arg: &str, kind_hint: Option<&str>, out_dir: &str) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    pb.set_message("resolving input");
    let input_json = input::resolve_to_json(input_arg).await?;

    pb.set_message("canonicalizing input");
    let canonical = signia_core::determinism::canonical_json::canonicalize_json(&input_json)?;

    pb.set_message("opening store");
    let store_cfg = signia_store::StoreConfig::local_dev(PathBuf::from(store_root))?;
    let store = signia_store::Store::open(store_cfg)?;

    pb.set_message("loading plugins");
    let mut reg = signia_plugins::registry::PluginRegistry::default();
    signia_plugins::builtin::repo::register(&mut reg);
    signia_plugins::builtin::dataset::register(&mut reg);
    signia_plugins::builtin::workflow::register(&mut reg);
    signia_plugins::builtin::api::register(&mut reg);
    signia_plugins::builtin::spec::register(&mut reg);

    pb.set_message("detecting kind");
    let detected = match kind_hint {
        Some("repo") => signia_plugins::builtin::config::schema_detect::DetectedKind::Repo,
        Some("dataset") => signia_plugins::builtin::config::schema_detect::DetectedKind::Dataset,
        Some("workflow") => signia_plugins::builtin::config::schema_detect::DetectedKind::Workflow,
        Some("openapi") => signia_plugins::builtin::config::schema_detect::DetectedKind::OpenApi,
        Some(_) => return Err(anyhow!("unknown kind hint")),
        None => signia_plugins::builtin::config::schema_detect::detect_input_kind(&canonical)?.kind,
    };

    let (kind_key, plugin_id) = match detected {
        signia_plugins::builtin::config::schema_detect::DetectedKind::Repo => ("repo", "builtin.repo"),
        signia_plugins::builtin::config::schema_detect::DetectedKind::Dataset => ("dataset", "builtin.dataset"),
        signia_plugins::builtin::config::schema_detect::DetectedKind::Workflow => ("workflow", "builtin.workflow"),
        signia_plugins::builtin::config::schema_detect::DetectedKind::OpenApi => ("openapi", "builtin.api.openapi"),
        signia_plugins::builtin::config::schema_detect::DetectedKind::Unknown => return Err(anyhow!("unable to detect input kind")),
    };

    pb.set_message("compiling");
    let mut ctx = signia_core::pipeline::context::PipelineContext::new(
        signia_core::pipeline::context::PipelineConfig::default(),
    );
    ctx.inputs.insert(kind_key.to_string(), canonical.clone());

    let plugin = reg.get(plugin_id).ok_or_else(|| anyhow!("plugin not found: {plugin_id}"))?;
    plugin.execute(&signia_plugins::plugin::PluginInput::Pipeline(&mut ctx))?;

    let ir_value = serde_json::to_value(&ctx.ir)?;
    let schema_json = signia_core::determinism::canonical_json::canonicalize_json(&ir_value)?;

    pb.set_message("storing artifacts");
    let schema_bytes = serde_json::to_vec(&schema_json)?;
    let schema_id = store.put_object_bytes(&schema_bytes)?;

    let manifest = export::build_manifest(&canonical, &schema_id, kind_key);
    let manifest_bytes = serde_json::to_vec(&manifest)?;
    let manifest_id = store.put_object_bytes(&manifest_bytes)?;

    let proof = export::build_proof(&canonical, &schema_id, &manifest_id)?;
    let proof_bytes = serde_json::to_vec(&proof)?;
    let proof_id = store.put_object_bytes(&proof_bytes)?;

    pb.set_message("writing bundle");
    export::write_bundle(out_dir, &schema_json, &manifest, &proof)?;

    pb.finish_and_clear();

    let out = CompileOut {
        kind: kind_key.to_string(),
        schema_id,
        manifest_id,
        proof_id,
        out_dir: out_dir.to_string(),
        metadata: ctx.metadata,
    };
    output::print(&out)?;
    Ok(())
}
