//! GitHub fetching interface for the built-in `repo` plugin.
//!
//! IMPORTANT: `signia-plugins` does NOT perform network I/O by default.
//! The SIGNIA host (CLI/API service) is responsible for fetching any remote
//! sources and providing deterministic, structured inputs to plugins.
//!
//! This file defines a *host adapter interface* for GitHub so the higher layers
//! can implement real network fetching (using `reqwest`, GitHub App auth, etc.)
//! without expanding the trusted surface area of `signia-core` / `signia-plugins`.
//!
//! What you get here:
//! - deterministic request/response structs
//! - strict limits to avoid accidental oversized inputs
//! - canonical conversion into the repo plugin's expected input shape
//!
//! What you do NOT get here:
//! - actual HTTP implementation
//! - token management or auth
//! - archive extraction
//!
//! Those belong in the host layer.

#![cfg(feature = "builtin")]

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use signia_core::determinism::hashing::hash_bytes_hex;
use signia_core::provenance::SourceRef;

/// Maximum number of files allowed by default when fetching a repo snapshot.
pub const DEFAULT_MAX_FILES: u64 = 10_000;

/// Maximum total bytes allowed by default when fetching a repo snapshot.
pub const DEFAULT_MAX_TOTAL_BYTES: u64 = 32 * 1024 * 1024;

/// Request describing what repo snapshot to fetch.
///
/// All fields are explicit; nothing is inferred from environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFetchRequest {
    /// Repository owner/org.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Git ref: commit SHA, tag, or branch name.
    ///
    /// For determinism, prefer a full commit SHA.
    pub git_ref: String,
    /// Optional subpath to scope the snapshot (e.g., "crates/signia-core").
    pub subpath: Option<String>,
    /// Maximum number of files.
    pub max_files: u64,
    /// Maximum total bytes across all files.
    pub max_total_bytes: u64,
    /// Include patterns (glob-like), interpreted by host.
    pub include: Vec<String>,
    /// Exclude patterns (glob-like), interpreted by host.
    pub exclude: Vec<String>,
    /// Whether file contents should be included.
    /// If false, the host may return metadata only.
    pub include_contents: bool,
    /// Extra host-specific options (must be deterministic).
    pub options: BTreeMap<String, String>,
}

impl GitHubFetchRequest {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>, git_ref: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            git_ref: git_ref.into(),
            subpath: None,
            max_files: DEFAULT_MAX_FILES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
            include: Vec::new(),
            exclude: Vec::new(),
            include_contents: false,
            options: BTreeMap::new(),
        }
    }

    pub fn with_subpath(mut self, subpath: impl Into<String>) -> Self {
        self.subpath = Some(subpath.into());
        self
    }

    pub fn with_limits(mut self, max_files: u64, max_total_bytes: u64) -> Self {
        self.max_files = max_files;
        self.max_total_bytes = max_total_bytes;
        self
    }

    pub fn with_include(mut self, pat: impl Into<String>) -> Self {
        self.include.push(pat.into());
        self
    }

    pub fn with_exclude(mut self, pat: impl Into<String>) -> Self {
        self.exclude.push(pat.into());
        self
    }

    pub fn with_option(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.options.insert(k.into(), v.into());
        self
    }

    /// Basic validation to ensure deterministic-friendly inputs.
    pub fn validate(&self) -> Result<()> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() || self.git_ref.trim().is_empty() {
            return Err(anyhow!("owner/repo/git_ref must be non-empty"));
        }
        if !self.owner.is_ascii() || !self.repo.is_ascii() || !self.git_ref.is_ascii() {
            return Err(anyhow!("owner/repo/git_ref must be ASCII"));
        }
        if self.max_files == 0 || self.max_total_bytes == 0 {
            return Err(anyhow!("max_files and max_total_bytes must be > 0"));
        }
        Ok(())
    }

    /// Convert to a deterministic SourceRef.
    pub fn to_source_ref(&self) -> SourceRef {
        // Locator is deterministic and does not include auth or machine-local paths.
        let mut locator = format!(
            "git:https://github.com/{}/{}.git#{}",
            self.owner, self.repo, self.git_ref
        );
        if let Some(s) = &self.subpath {
            locator.push(':');
            locator.push_str(s);
        }
        SourceRef::new("git", locator).with_revision(self.git_ref.clone())
    }
}

/// A deterministic representation of a repository file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoFile {
    /// Path relative to repo root (or request subpath).
    pub path: String,
    /// File size in bytes.
    pub size: u64,
    /// Content hash (sha256 hex) if content available.
    pub sha256: Option<String>,
    /// Optional file mode string ("100644", etc.) if provided.
    pub mode: Option<String>,
    /// Optional raw bytes (host-provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}

impl RepoFile {
    pub fn new(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            size,
            sha256: None,
            mode: None,
            bytes: None,
        }
    }
}

/// A repo snapshot returned by the host fetcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub source: SourceRef,
    pub files: Vec<RepoFile>,
    /// Deterministic digest over the snapshot metadata.
    pub snapshot_hash: String,
}

impl RepoSnapshot {
    /// Build a deterministic digest over file metadata + optional content hashes.
    pub fn compute_snapshot_hash(files: &[RepoFile]) -> Result<String> {
        // Stable concatenation format:
        // path \t size \t sha256? \n
        let mut buf = Vec::new();
        for f in files {
            buf.extend_from_slice(f.path.as_bytes());
            buf.extend_from_slice(b"\t");
            buf.extend_from_slice(f.size.to_string().as_bytes());
            buf.extend_from_slice(b"\t");
            if let Some(h) = &f.sha256 {
                buf.extend_from_slice(h.as_bytes());
            }
            buf.extend_from_slice(b"\n");
        }
        Ok(hash_bytes_hex(&buf)?)
    }

    /// Convert snapshot to the structured JSON input expected by the builtin repo plugin.
    ///
    /// Shape:
    /// {
    ///   "name": "<owner>/<repo>",
    ///   "ref": "<git_ref>",
    ///   "source": { ... },
    ///   "snapshotHash": "<sha256>",
    ///   "files": [ { "path": "...", "size": 123, "sha256": "..." } ]
    /// }
    pub fn to_repo_plugin_input(&self, owner: &str, repo: &str, git_ref: &str) -> serde_json::Value {
        let files = self
            .files
            .iter()
            .map(|f| {
                let mut o = serde_json::Map::new();
                o.insert("path".to_string(), serde_json::Value::String(f.path.clone()));
                o.insert("size".to_string(), serde_json::Value::Number(serde_json::Number::from(f.size)));
                if let Some(h) = &f.sha256 {
                    o.insert("sha256".to_string(), serde_json::Value::String(h.clone()));
                }
                if let Some(m) = &f.mode {
                    o.insert("mode".to_string(), serde_json::Value::String(m.clone()));
                }
                serde_json::Value::Object(o)
            })
            .collect::<Vec<_>>();

        json!({
            "name": format!("{}/{}", owner, repo),
            "ref": git_ref,
            "source": {
                "type": self.source.r#type,
                "locator": self.source.locator,
                "digest": self.source.digest,
                "revision": self.source.revision,
                "subpath": self.source.subpath,
                "extras": self.source.extras,
            },
            "snapshotHash": self.snapshot_hash,
            "files": files
        })
    }
}

/// Trait implemented by the host to fetch GitHub repositories deterministically.
///
/// The host decides:
/// - authentication mechanism
/// - rate limiting
/// - caching
/// - archive extraction and filtering
///
/// This crate only defines the interface.
pub trait GitHubFetcher: Send + Sync {
    /// Fetch a repo snapshot for the given request.
    ///
    /// MUST:
    /// - respect request limits (max_files, max_total_bytes)
    /// - apply include/exclude filters deterministically
    /// - normalize paths deterministically
    /// - avoid embedding machine-local paths in results
    fn fetch_repo_snapshot(&self, req: &GitHubFetchRequest) -> Result<RepoSnapshot>;
}

/// A default fetcher that always fails (used when networking is disabled).
pub struct NoNetworkGitHubFetcher;

impl GitHubFetcher for NoNetworkGitHubFetcher {
    fn fetch_repo_snapshot(&self, _req: &GitHubFetchRequest) -> Result<RepoSnapshot> {
        Err(anyhow!(
            "GitHub fetch is not available in signia-plugins; implement GitHubFetcher in the host layer"
        ))
    }
}

/// Build a RepoSnapshot from a file list that the host already materialized.
///
/// This helper computes content hashes if `bytes` is present and validates limits.
pub fn snapshot_from_files(req: &GitHubFetchRequest, mut files: Vec<RepoFile>) -> Result<RepoSnapshot> {
    req.validate()?;

    if files.len() as u64 > req.max_files {
        return Err(anyhow!(
            "repo file count exceeds limit: files={}, max_files={}",
            files.len(),
            req.max_files
        ));
    }

    let mut total = 0u64;
    for f in &mut files {
        // If bytes provided, compute sha256 deterministically.
        if let Some(b) = &f.bytes {
            total = total.saturating_add(b.len() as u64);
            if f.sha256.is_none() {
                f.sha256 = Some(hash_bytes_hex(b)?);
            }
            // Prefer reported size to match bytes if present.
            f.size = b.len() as u64;
        } else {
            total = total.saturating_add(f.size);
        }
    }

    if total > req.max_total_bytes {
        return Err(anyhow!(
            "repo total size exceeds limit: total_bytes={}, max_total_bytes={}",
            total,
            req.max_total_bytes
        ));
    }

    // Compute a deterministic snapshot hash based on file metadata/hashes.
    // Host should provide stable file ordering; however, to be safe, we sort by path here.
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let snapshot_hash = RepoSnapshot::compute_snapshot_hash(&files)?;
    let source = req.to_source_ref();

    Ok(RepoSnapshot {
        source,
        files,
        snapshot_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_hash_stable() {
        let req = GitHubFetchRequest::new("o", "r", "deadbeef").with_limits(10, 1024);
        let files = vec![
            RepoFile { path: "b".to_string(), size: 1, sha256: Some("x".to_string()), mode: None, bytes: None },
            RepoFile { path: "a".to_string(), size: 2, sha256: Some("y".to_string()), mode: None, bytes: None },
        ];

        let s1 = snapshot_from_files(&req, files.clone()).unwrap();
        let s2 = snapshot_from_files(&req, files).unwrap();
        assert_eq!(s1.snapshot_hash, s2.snapshot_hash);
    }

    #[test]
    fn rejects_limits() {
        let req = GitHubFetchRequest::new("o", "r", "deadbeef").with_limits(1, 10);
        let files = vec![
            RepoFile::new("a", 1),
            RepoFile::new("b", 1),
        ];
        assert!(snapshot_from_files(&req, files).is_err());
    }
}
