//! Version helpers.
//!
//! This module centralizes version parsing and validation for SIGNIA artifacts.
//! It is intentionally strict and returns stable error codes for invalid versions.

use crate::errors::{SigniaError, SigniaResult};

/// Known schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaVersion {
    V1,
}

impl SchemaVersion {
    /// Parse a schema version string (e.g. "v1").
    pub fn parse(s: &str) -> SigniaResult<Self> {
        match s {
            "v1" => Ok(Self::V1),
            _ => Err(SigniaError::invalid_argument(format!(
                "unsupported schema version: {s}"
            ))),
        }
    }

    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }
}

/// Known manifest versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVersion {
    V1,
}

impl ManifestVersion {
    /// Parse a manifest version string (e.g. "v1").
    pub fn parse(s: &str) -> SigniaResult<Self> {
        match s {
            "v1" => Ok(Self::V1),
            _ => Err(SigniaError::invalid_argument(format!(
                "unsupported manifest version: {s}"
            ))),
        }
    }

    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }
}

/// Known proof versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofVersion {
    V1,
}

impl ProofVersion {
    /// Parse a proof version string (e.g. "v1").
    pub fn parse(s: &str) -> SigniaResult<Self> {
        match s {
            "v1" => Ok(Self::V1),
            _ => Err(SigniaError::invalid_argument(format!(
                "unsupported proof version: {s}"
            ))),
        }
    }

    /// Return the canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }
}

/// Validate that a version field matches the expected version.
pub fn require_version(actual: &str, expected: &str, field: &str) -> SigniaResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(SigniaError::invalid_argument(format!(
            "invalid {field}: expected {expected}, got {actual}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_schema_v1() {
        assert_eq!(SchemaVersion::parse("v1").unwrap(), SchemaVersion::V1);
    }

    #[test]
    fn parse_schema_unknown() {
        let e = SchemaVersion::parse("v9").unwrap_err();
        assert!(format!("{e:?}").contains("unsupported schema version"));
    }

    #[test]
    fn require_version_ok() {
        require_version("v1", "v1", "version").unwrap();
    }

    #[test]
    fn require_version_err() {
        let e = require_version("v2", "v1", "version").unwrap_err();
        assert!(format!("{e:?}").contains("expected v1"));
    }
}
