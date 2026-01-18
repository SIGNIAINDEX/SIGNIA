//! Built-in configuration module.
//!
//! This module defines configuration structures for SIGNIA built-in plugins and
//! host-facing behavior.
//!
//! Goals:
//! - Provide a single place for default limits and behavior toggles.
//! - Keep configuration deterministic and serializable.
//! - Avoid direct I/O. Loading config from files/env is handled by the host.
//!
//! Conventions:
//! - All limits are explicit.
//! - Defaults are conservative.
//! - All structs derive Serialize/Deserialize for easy config handling.

#![cfg(feature = "builtin")]

use serde::{Deserialize, Serialize};

/// Built-in configuration root.
///
/// Hosts can embed this config and allow users to override fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinConfig {
    #[serde(default)]
    pub repo: RepoConfig,
    #[serde(default)]
    pub dataset: DatasetConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub api: ApiConfig,
}

impl Default for BuiltinConfig {
    fn default() -> Self {
        Self {
            repo: RepoConfig::default(),
            dataset: DatasetConfig::default(),
            workflow: WorkflowConfig::default(),
            api: ApiConfig::default(),
        }
    }
}

/// Repository plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    /// Maximum number of files to accept in a repository snapshot.
    #[serde(default = "RepoConfig::default_max_files")]
    pub max_files: usize,

    /// Maximum total bytes allowed across all file contents provided to the pipeline.
    #[serde(default = "RepoConfig::default_max_total_bytes")]
    pub max_total_bytes: u64,

    /// Maximum individual file size in bytes.
    #[serde(default = "RepoConfig::default_max_file_bytes")]
    pub max_file_bytes: u64,

    /// Include patterns (glob-like). If empty, include all.
    #[serde(default)]
    pub include: Vec<String>,

    /// Exclude patterns (glob-like).
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Whether to allow binary files. If false, binary files must be omitted by the host.
    #[serde(default)]
    pub allow_binary: bool,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            max_files: Self::default_max_files(),
            max_total_bytes: Self::default_max_total_bytes(),
            max_file_bytes: Self::default_max_file_bytes(),
            include: Vec::new(),
            exclude: vec![
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
            ],
            allow_binary: false,
        }
    }
}

impl RepoConfig {
    fn default_max_files() -> usize {
        50_000
    }
    fn default_max_total_bytes() -> u64 {
        256 * 1024 * 1024 // 256 MiB
    }
    fn default_max_file_bytes() -> u64 {
        8 * 1024 * 1024 // 8 MiB
    }
}

/// Dataset plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    /// Maximum number of files in a dataset.
    #[serde(default = "DatasetConfig::default_max_files")]
    pub max_files: usize,

    /// Maximum total bytes across provided file contents.
    #[serde(default = "DatasetConfig::default_max_total_bytes")]
    pub max_total_bytes: u64,

    /// Whether to compute Merkle roots in addition to fingerprints.
    #[serde(default)]
    pub enable_merkle: bool,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            max_files: Self::default_max_files(),
            max_total_bytes: Self::default_max_total_bytes(),
            enable_merkle: true,
        }
    }
}

impl DatasetConfig {
    fn default_max_files() -> usize {
        100_000
    }
    fn default_max_total_bytes() -> u64 {
        512 * 1024 * 1024 // 512 MiB
    }
}

/// Workflow plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Maximum nodes allowed in a workflow graph.
    #[serde(default = "WorkflowConfig::default_max_nodes")]
    pub max_nodes: usize,

    /// Maximum edges allowed in a workflow graph.
    #[serde(default = "WorkflowConfig::default_max_edges")]
    pub max_edges: usize,

    /// Whether YAML parsing is enabled in hosts that support YAML.
    #[serde(default)]
    pub enable_yaml: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_nodes: Self::default_max_nodes(),
            max_edges: Self::default_max_edges(),
            enable_yaml: true,
        }
    }
}

impl WorkflowConfig {
    fn default_max_nodes() -> usize {
        200_000
    }
    fn default_max_edges() -> usize {
        400_000
    }
}

/// Built-in API configuration for hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether to expose the built-in catalog endpoints.
    #[serde(default = "ApiConfig::default_enabled")]
    pub enabled: bool,

    /// API version string (pure metadata).
    #[serde(default = "ApiConfig::default_version")]
    pub version: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            version: Self::default_version(),
        }
    }
}

impl ApiConfig {
    fn default_enabled() -> bool {
        true
    }
    fn default_version() -> String {
        "v1".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = BuiltinConfig::default();
        assert!(c.repo.max_files > 0);
        assert!(c.dataset.max_files > 0);
        assert!(c.workflow.max_nodes > 0);
        assert!(c.api.enabled);
    }

    #[test]
    fn serde_roundtrip() {
        let c = BuiltinConfig::default();
        let s = serde_json::to_string(&c).unwrap();
        let d: BuiltinConfig = serde_json::from_str(&s).unwrap();
        assert_eq!(d.api.version, "v1");
    }
}
