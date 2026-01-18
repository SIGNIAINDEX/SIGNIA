//! Built-in plugin specification registry.
//!
//! This module provides a deterministic "catalog" of built-in plugin specs,
//! independent from any host configuration.
//!
//! Why this exists:
//! - UIs and CLIs can query supported builtins without instantiating plugins.
//! - Specs can be used for validation, docs, and compatibility checks.
//! - Registration can be centralized for tests and simple hosts.
//!
//! Notes:
//! - This file does not do network or filesystem I/O.
//! - Actual plugin implementations live under `crate::builtin::*`.

#![cfg(feature = "builtin")]

use crate::registry::PluginRegistry;
use crate::spec::PluginSpec;

/// Built-in plugin ids shipped with this crate.
///
/// Keep this list stable and append-only when possible.
pub const BUILTIN_PLUGIN_IDS: [&str; 2] = ["builtin.repo", "builtin.dataset"];

/// Return deterministic specs for all built-in plugins.
///
/// This is intentionally a pure function so it can be used in `--json` outputs
/// and documentation generators.
pub fn builtin_specs() -> Vec<PluginSpec> {
    vec![repo_spec(), dataset_spec()]
}

/// Register all built-in plugins into the provided registry.
pub fn register_all(registry: &mut PluginRegistry) {
    crate::builtin::repo::register(registry);
    crate::builtin::dataset::register(registry);
}

/// Spec for `builtin.repo`.
pub fn repo_spec() -> PluginSpec {
    PluginSpec::new("builtin.repo", "Repository Plugin", "0.1.0")
        .support("repo")
        .limit("max_nodes", 200_000)
        .limit("max_edges", 400_000)
        .want("network", false)
        .want("filesystem", false)
        .meta("category", "source")
}

/// Spec for `builtin.dataset`.
pub fn dataset_spec() -> PluginSpec {
    PluginSpec::new("builtin.dataset", "Dataset Plugin", "0.1.0")
        .support("dataset")
        .limit("max_nodes", 300_000)
        .limit("max_edges", 600_000)
        .want("network", false)
        .want("filesystem", false)
        .meta("category", "data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_are_deterministic() {
        let a = builtin_specs();
        let b = builtin_specs();
        assert_eq!(a.len(), b.len());
        assert_eq!(a[0].id, b[0].id);
        assert_eq!(a[1].id, b[1].id);
    }

    #[test]
    fn ids_match_specs() {
        let specs = builtin_specs();
        let ids: Vec<&str> = specs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, BUILTIN_PLUGIN_IDS);
    }
}
