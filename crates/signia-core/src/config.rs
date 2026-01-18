//! Configuration structures for signia-core.
//!
//! This module defines explicit, serializable configuration objects used by
//! higher-level components (CLI, API, plugins) to control determinism,
//! normalization, hashing, and limits.
//!
//! The core crate itself does not read environment variables. All configuration
//! must be provided explicitly by the caller to preserve determinism.

use crate::errors::{SigniaError, SigniaResult};

/// Global configuration container.
#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub normalization: NormalizationConfig,
    pub hashing: HashingConfig,
    pub limits: LimitsConfig,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            normalization: NormalizationConfig::default(),
            hashing: HashingConfig::default(),
            limits: LimitsConfig::default(),
        }
    }
}

/// Normalization-related configuration.
#[derive(Debug, Clone)]
pub struct NormalizationConfig {
    pub path_root: String,
    pub newline: NewlineMode,
    pub encoding: Encoding,
    pub symlink_policy: SymlinkPolicy,
    pub network_policy: NetworkPolicy,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            path_root: "artifact:/".to_string(),
            newline: NewlineMode::Lf,
            encoding: Encoding::Utf8,
            symlink_policy: SymlinkPolicy::Deny,
            network_policy: NetworkPolicy::Deny,
        }
    }
}

/// Hashing-related configuration.
#[derive(Debug, Clone)]
pub struct HashingConfig {
    pub algorithm: HashAlgorithm,
    pub domain: String,
}

impl Default for HashingConfig {
    fn default() -> Self {
        Self {
            algorithm: HashAlgorithm::Sha256,
            domain: "signia.v1".to_string(),
        }
    }
}

/// Resource and complexity limits.
#[derive(Debug, Clone)]
pub struct LimitsConfig {
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
    pub max_files: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub timeout_ms: u64,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_total_bytes: 512 * 1024 * 1024,
            max_file_bytes: 64 * 1024 * 1024,
            max_files: 100_000,
            max_depth: 128,
            max_nodes: 1_000_000,
            max_edges: 2_000_000,
            timeout_ms: 60_000,
        }
    }
}

/// Supported newline normalization modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineMode {
    Lf,
}

impl NewlineMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "lf",
        }
    }
}

/// Supported encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
}

impl Encoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
        }
    }
}

/// Symlink handling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkPolicy {
    Deny,
    ResolveWithinRoot,
}

impl SymlinkPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::ResolveWithinRoot => "resolve-within-root",
        }
    }
}

/// Network access policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPolicy {
    Deny,
    AllowPinnedOnly,
}

impl NetworkPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::AllowPinnedOnly => "allow-pinned-only",
        }
    }
}

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

impl HashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Blake3 => "blake3",
        }
    }
}

/// Validate a full configuration object.
pub fn validate_config(cfg: &CoreConfig) -> SigniaResult<()> {
    if cfg.limits.max_file_bytes > cfg.limits.max_total_bytes {
        return Err(SigniaError::invalid_argument(
            "max_file_bytes must not exceed max_total_bytes",
        ));
    }

    if cfg.limits.max_nodes == 0 {
        return Err(SigniaError::invalid_argument(
            "max_nodes must be greater than zero",
        ));
    }

    if cfg.hashing.domain.is_empty() {
        return Err(SigniaError::invalid_argument(
            "hashing domain must not be empty",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = CoreConfig::default();
        validate_config(&cfg).unwrap();
    }

    #[test]
    fn invalid_limits_detected() {
        let mut cfg = CoreConfig::default();
        cfg.limits.max_file_bytes = cfg.limits.max_total_bytes + 1;
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn empty_domain_detected() {
        let mut cfg = CoreConfig::default();
        cfg.hashing.domain = "".to_string();
        assert!(validate_config(&cfg).is_err());
    }
}
