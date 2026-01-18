//! Plugin registry and resolution for SIGNIA.
//!
//! The registry stores available plugins and provides deterministic resolution.
//!
//! Requirements:
//! - stable ordering for lookups and iteration
//! - clear errors for missing/ambiguous plugins
//! - no global mutable state
//!
//! The registry does not execute plugins; it only stores metadata and instances.

use std::collections::BTreeMap;

use crate::plugin::{Plugin, PluginVersion};
use crate::spec::{evaluate_spec, PluginId, PluginSpec, SpecEvaluation};

/// A plugin instance plus its static spec.
pub struct RegisteredPlugin {
    pub spec: PluginSpec,
    pub plugin: Box<dyn Plugin>,
}

/// A registry of plugins keyed by plugin id.
pub struct PluginRegistry {
    plugins: BTreeMap<String, RegisteredPlugin>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
        }
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns true if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Register a plugin instance with its spec.
    ///
    /// Registration order does not affect deterministic resolution because the
    /// internal store is a `BTreeMap`.
    pub fn register(&mut self, spec: PluginSpec, plugin: Box<dyn Plugin>) -> anyhow::Result<()> {
        spec.validate()?;

        let id = spec.id.as_str().to_string();
        if self.plugins.contains_key(&id) {
            anyhow::bail!("plugin id already registered: {id}");
        }

        self.plugins.insert(id, RegisteredPlugin { spec, plugin });
        Ok(())
    }

    /// Get a plugin by id.
    pub fn get(&self, id: &str) -> Option<&RegisteredPlugin> {
        self.plugins.get(id)
    }

    /// List plugin ids in deterministic order.
    pub fn list_ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Iterate over registered plugins in deterministic id order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RegisteredPlugin)> {
        self.plugins.iter()
    }
}

/// Plugin resolution request.
#[derive(Debug, Clone)]
pub struct PluginResolver {
    /// Host capabilities used to decide whether a plugin can run.
    pub host: crate::plugin::HostCapabilities,
}

impl PluginResolver {
    pub fn new(host: crate::plugin::HostCapabilities) -> Self {
        Self { host }
    }

    /// Resolve a plugin by id and optional version constraint.
    ///
    /// Resolution strategy:
    /// - require exact id match
    /// - if version constraint provided, require plugin.version() to match
    /// - evaluate host capability compatibility via PluginSpec wants
    pub fn resolve(
        &self,
        registry: &PluginRegistry,
        id: &str,
        version: Option<PluginVersion>,
    ) -> anyhow::Result<ResolvedPlugin<'_>> {
        let reg = registry
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("plugin not found: {id}"))?;

        if let Some(v) = version {
            if reg.plugin.version() != v.0 {
                anyhow::bail!(
                    "plugin version mismatch for {id}: requested={}, available={}",
                    v.0,
                    reg.plugin.version()
                );
            }
        }

        let ev = evaluate_spec(&reg.spec, &self.host);
        if !ev.allowed {
            let reason = ev.reason.unwrap_or_else(|| "denied".to_string());
            anyhow::bail!(
                "plugin {id} is not allowed under host capabilities: {reason}; missing={:?}",
                ev.missing
            );
        }

        Ok(ResolvedPlugin {
            id: PluginId::new(id),
            spec: &reg.spec,
            plugin: reg.plugin.as_ref(),
            evaluation: ev,
        })
    }
}

/// A resolved plugin reference.
pub struct ResolvedPlugin<'a> {
    pub id: PluginId,
    pub spec: &'a PluginSpec,
    pub plugin: &'a dyn Plugin,
    pub evaluation: SpecEvaluation,
}

impl<'a> ResolvedPlugin<'a> {
    pub fn name(&self) -> &str {
        &self.spec.name
    }

    pub fn version(&self) -> &str {
        self.plugin.version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{HostCapabilities, PluginInput, PluginOutput};

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test"
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn supports(&self, input_type: &str) -> bool {
            input_type == "x"
        }
        fn execute(&self, _input: &PluginInput) -> crate::plugin::PluginResult<PluginOutput> {
            Ok(PluginOutput::None)
        }
    }

    #[test]
    fn registry_register_and_resolve() {
        let mut reg = PluginRegistry::new();
        let spec = PluginSpec::new("builtin.test", "Test", "0.1.0")
            .support("x")
            .want("network", false);

        reg.register(spec, Box::new(TestPlugin)).unwrap();

        let resolver = PluginResolver::new(HostCapabilities {
            network: false,
            filesystem: false,
            clock: false,
            spawn: false,
        });

        let resolved = resolver.resolve(&reg, "builtin.test", None).unwrap();
        assert_eq!(resolved.version(), "0.1.0");
    }
}
