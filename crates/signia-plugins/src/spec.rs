//! Plugin specification types for SIGNIA.
//!
//! This module defines stable, serializable "spec" structures that describe:
//! - plugin identity and capabilities
//! - supported input types and versions
//! - declared limits and permissions
//!
//! Specs are used for:
//! - registry metadata
//! - manifest declarations
//! - auditing and UI
//! - deterministic selection and resolution
//!
//! Specs are data-only and MUST NOT execute code.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::plugin::HostCapabilities;

/// Stable plugin identifier.
///
/// Format recommendations:
/// - lowercase
/// - ASCII
/// - segments separated by dots
/// - example: "builtin.repo"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(pub String);

impl PluginId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Plugin capability description.
///
/// This is the static declaration of what a plugin can do.
/// It is not a permission grant: the host decides what is allowed.
#[derive(Debug, Clone, Default)]
pub struct PluginSpec {
    /// Stable plugin id.
    pub id: PluginId,

    /// Human-readable display name.
    pub name: String,

    /// Plugin semantic version string (host interprets).
    pub version: String,

    /// Input types supported by this plugin (e.g., "repo", "openapi", "dataset").
    pub supports: Vec<String>,

    /// Optional supported versions per input type (e.g., "repo:v1").
    pub supports_versions: BTreeMap<String, Vec<String>>,

    /// Declared limits (nodes, edges, bytes).
    pub limits: BTreeMap<String, u64>,

    /// Declared permissions desired by plugin (host grants or denies).
    ///
    /// Examples:
    /// - "network" = false
    /// - "filesystem" = false
    /// - "clock" = false
    pub wants: BTreeMap<String, bool>,

    /// Arbitrary metadata for UI.
    pub meta: BTreeMap<String, String>,
}

impl PluginSpec {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: PluginId::new(id),
            name: name.into(),
            version: version.into(),
            supports: Vec::new(),
            supports_versions: BTreeMap::new(),
            limits: BTreeMap::new(),
            wants: BTreeMap::new(),
            meta: BTreeMap::new(),
        }
    }

    pub fn support(mut self, input_type: impl Into<String>) -> Self {
        self.supports.push(input_type.into());
        self
    }

    pub fn support_version(mut self, input_type: impl Into<String>, version: impl Into<String>) -> Self {
        let k = input_type.into();
        self.supports_versions.entry(k).or_default().push(version.into());
        self
    }

    pub fn limit(mut self, key: impl Into<String>, value: u64) -> Self {
        self.limits.insert(key.into(), value);
        self
    }

    pub fn want(mut self, key: impl Into<String>, value: bool) -> Self {
        self.wants.insert(key.into(), value);
        self
    }

    pub fn meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    /// Returns true if spec declares support for input type.
    pub fn supports_type(&self, input_type: &str) -> bool {
        self.supports.iter().any(|t| t == input_type)
    }

    /// Validate spec for basic quality constraints.
    pub fn validate(&self) -> Result<()> {
        if self.id.as_str().trim().is_empty() {
            anyhow::bail!("plugin id is empty");
        }
        if !self.id.as_str().is_ascii() {
            anyhow::bail!("plugin id must be ASCII");
        }
        if self.name.trim().is_empty() {
            anyhow::bail!("plugin name is empty");
        }
        if self.version.trim().is_empty() {
            anyhow::bail!("plugin version is empty");
        }
        Ok(())
    }
}

/// Host policy evaluation for a spec.
#[derive(Debug, Clone)]
pub struct SpecEvaluation {
    /// Whether the plugin can run under provided host capabilities.
    pub allowed: bool,

    /// Denial reason (if not allowed).
    pub reason: Option<String>,

    /// Capabilities required by plugin and missing from host.
    pub missing: Vec<String>,
}

impl SpecEvaluation {
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            missing: Vec::new(),
        }
    }

    pub fn denied(reason: impl Into<String>, missing: Vec<String>) -> Self {
        Self {
            allowed: false,
            reason: Some(reason.into()),
            missing,
        }
    }
}

/// Evaluate whether a plugin spec is compatible with host capabilities.
///
/// This is a conservative check based on `wants`.
pub fn evaluate_spec(spec: &PluginSpec, host: &HostCapabilities) -> SpecEvaluation {
    let mut missing = Vec::new();

    for (k, wants) in &spec.wants {
        if !*wants {
            continue;
        }
        let ok = match k.as_str() {
            "network" => host.network,
            "filesystem" => host.filesystem,
            "clock" => host.clock,
            "spawn" => host.spawn,
            _ => false,
        };
        if !ok {
            missing.push(k.clone());
        }
    }

    if missing.is_empty() {
        SpecEvaluation::allowed()
    } else {
        SpecEvaluation::denied("host capabilities do not satisfy plugin wants", missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_validate_ok() {
        let s = PluginSpec::new("builtin.repo", "Repo", "0.1.0").support("repo");
        s.validate().unwrap();
    }

    #[test]
    fn evaluate_spec_denies_missing() {
        let s = PluginSpec::new("x", "X", "0.1.0").want("network", true);
        let host = HostCapabilities {
            network: false,
            filesystem: false,
            clock: false,
            spawn: false,
        };
        let ev = evaluate_spec(&s, &host);
        assert!(!ev.allowed);
        assert_eq!(ev.missing, vec!["network".to_string()]);
    }
}
