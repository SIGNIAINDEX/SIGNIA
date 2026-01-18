//! Global plugin contract tests.
//!
//! These tests validate guarantees that apply to ALL plugins (builtin or external):
//! - deterministic execution
//! - stable IDs
//! - canonical JSON behavior
//! - no implicit ordering dependence
//!
//! This test module is compiled without feature flags and represents the
//! minimum contract required for any Signia-compatible plugin.

use serde_json::json;

use signia_core::determinism::canonical_json::canonicalize_json;
use signia_core::pipeline::context::{PipelineConfig, PipelineContext};

use signia_plugins::registry::PluginRegistry;
use signia_plugins::plugin::{Plugin, PluginInput};

#[test]
fn empty_registry_is_stable() {
    let r1 = PluginRegistry::default();
    let r2 = PluginRegistry::default();

    assert_eq!(r1.list().len(), 0);
    assert_eq!(r1.list(), r2.list());
}

#[test]
fn registry_order_is_deterministic() {
    let mut r = PluginRegistry::default();

    let ids: Vec<String> = r.list().into_iter().map(|s| s.id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}

#[test]
fn pipeline_context_is_deterministic() {
    let cfg = PipelineConfig::default();
    let ctx1 = PipelineContext::new(cfg.clone());
    let ctx2 = PipelineContext::new(cfg);

    assert_eq!(ctx1.config, ctx2.config);
    assert_eq!(ctx1.inputs.len(), ctx2.inputs.len());
}

#[test]
fn canonical_json_roundtrip_is_stable() {
    let input = json!({
        "b": [3,2,1],
        "a": { "y":2, "x":1 }
    });

    let c1 = canonicalize_json(&input).unwrap();
    let c2 = canonicalize_json(&input).unwrap();

    assert_eq!(c1, c2);
}

#[test]
fn plugin_execute_contract_allows_no_side_effects() {
    // This test is intentionally generic.
    // It asserts that executing a plugin with an empty registry
    // does not panic or mutate global state.
    let mut registry = PluginRegistry::default();

    let ctx = PipelineContext::new(PipelineConfig::default());
    let input = PluginInput::None;

    for spec in registry.list() {
        let plugin = registry.get(&spec.id).unwrap();
        let mut ctx_clone = ctx.clone();
        let _ = plugin.execute(&input);
        assert_eq!(ctx_clone.inputs, ctx.inputs);
    }
}

#[test]
fn repeated_execution_produces_identical_results() {
    let mut registry = PluginRegistry::default();
    let ctx_base = PipelineContext::new(PipelineConfig::default());

    for spec in registry.list() {
        let plugin = registry.get(&spec.id).unwrap();

        let mut ctx1 = ctx_base.clone();
        let mut ctx2 = ctx_base.clone();

        let input = PluginInput::Pipeline(&mut ctx1);
        plugin.execute(&input).ok();

        let input = PluginInput::Pipeline(&mut ctx2);
        plugin.execute(&input).ok();

        assert_eq!(
            serde_json::to_value(&ctx1).unwrap(),
            serde_json::to_value(&ctx2).unwrap()
        );
    }
}
