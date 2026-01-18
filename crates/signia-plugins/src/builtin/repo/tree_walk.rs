//! Deterministic repo tree walking helpers for the built-in `repo` plugin.
//!
//! IMPORTANT:
//! - Plugins must not access the filesystem directly.
//! - The host (CLI/API) may use these helpers to produce structured inputs.
//!
//! This module provides:
//! - stable path normalization
//! - deterministic include/exclude filtering (glob-like)
//! - deterministic ordering
//! - limits enforcement
//!
//! It operates on an in-memory "virtual tree" representation so it is usable
//! both for local filesystem snapshots and remote (GitHub) snapshots.

#![cfg(feature = "builtin")]

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use crate::builtin::repo::github_fetch::{RepoFile, DEFAULT_MAX_FILES, DEFAULT_MAX_TOTAL_BYTES};

/// Virtual file entry for deterministic walking.
#[derive(Debug, Clone)]
pub struct VFile {
    pub path: String,
    pub bytes: Option<Vec<u8>>,
    pub size: u64,
    pub mode: Option<String>,
    pub meta: BTreeMap<String, String>,
}

impl VFile {
    pub fn new(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            bytes: None,
            size,
            mode: None,
            meta: BTreeMap::new(),
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.size = bytes.len() as u64;
        self.bytes = Some(bytes);
        self
    }
}

/// Tree-walk options.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub max_files: u64,
    pub max_total_bytes: u64,
    pub include_contents: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            max_files: DEFAULT_MAX_FILES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
            include_contents: false,
        }
    }
}

/// Normalize a path deterministically:
/// - replace backslashes with forward slashes
/// - remove repeated slashes
/// - remove leading "./"
/// - reject ".." segments (to avoid traversal ambiguity)
pub fn normalize_repo_path(p: &str) -> Result<String> {
    let mut s = p.replace('\\', "/");

    while s.contains("//") {
        s = s.replace("//", "/");
    }
    while s.starts_with("./") {
        s = s.trim_start_matches("./").to_string();
    }
    if s.starts_with('/') {
        s = s.trim_start_matches('/').to_string();
    }

    for seg in s.split('/') {
        if seg == ".." {
            return Err(anyhow!("path contains '..' segment: {p}"));
        }
    }
    Ok(s)
}

/// Very small deterministic "glob-like" matcher.
///
/// Supported forms:
/// - "*" matches any sequence within a segment
/// - "**" matches across path separators
/// - suffix match like "*.rs"
///
/// This is not a full glob engine but is deterministic and sufficient for common filters.
pub fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == "**" {
        return true;
    }

    // Quick path: exact match
    if path == pattern {
        return true;
    }

    // Split on "**"
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        let mut idx = 0usize;
        for part in parts {
            if part.is_empty() {
                continue;
            }
            if let Some(pos) = path[idx..].find(part) {
                idx += pos + part.len();
            } else {
                return false;
            }
        }
        return true;
    }

    // Segment-level "*"
    if pattern.contains('*') {
        // Convert to a simple regex-like matching without allocating regex engine.
        // Only supports "*" wildcard.
        let mut pi = 0usize;
        let pbytes = pattern.as_bytes();
        let sbytes = path.as_bytes();

        let mut si = 0usize;
        let mut star: Option<usize> = None;
        let mut match_i = 0usize;

        while si < sbytes.len() {
            if pi < pbytes.len() && (pbytes[pi] == sbytes[si]) {
                pi += 1;
                si += 1;
                continue;
            }

            if pi < pbytes.len() && pbytes[pi] == b'*' {
                star = Some(pi);
                pi += 1;
                match_i = si;
                continue;
            }

            if let Some(st) = star {
                pi = st + 1;
                match_i += 1;
                si = match_i;
                continue;
            }

            return false;
        }

        while pi < pbytes.len() && pbytes[pi] == b'*' {
            pi += 1;
        }
        return pi == pbytes.len();
    }

    false
}

/// Determine whether a path is included given include/exclude lists.
/// Deterministic rules:
/// - If include is empty: include all
/// - If include is non-empty: include if any include pattern matches
/// - Exclude always removes if any exclude pattern matches
pub fn is_included(path: &str, include: &[String], exclude: &[String]) -> bool {
    let inc_ok = if include.is_empty() {
        true
    } else {
        include.iter().any(|p| matches_pattern(path, p))
    };

    if !inc_ok {
        return false;
    }

    let exc = exclude.iter().any(|p| matches_pattern(path, p));
    !exc
}

/// Walk a set of virtual files deterministically, applying filters and limits.
///
/// Output is a `Vec<RepoFile>` sorted by normalized path.
pub fn walk_virtual_files(files: &[VFile], opts: &WalkOptions) -> Result<Vec<RepoFile>> {
    let mut selected: Vec<(String, &VFile)> = Vec::new();

    for f in files {
        let norm = normalize_repo_path(&f.path)?;
        if is_included(&norm, &opts.include, &opts.exclude) {
            selected.push((norm, f));
        }
    }

    selected.sort_by(|a, b| a.0.cmp(&b.0));

    if selected.len() as u64 > opts.max_files {
        return Err(anyhow!(
            "file count exceeds limit: files={}, max_files={}",
            selected.len(),
            opts.max_files
        ));
    }

    let mut total = 0u64;
    let mut out = Vec::with_capacity(selected.len());

    for (path, f) in selected {
        let size = if let Some(b) = &f.bytes {
            b.len() as u64
        } else {
            f.size
        };

        total = total.saturating_add(size);
        if total > opts.max_total_bytes {
            return Err(anyhow!(
                "total bytes exceeds limit: total_bytes={}, max_total_bytes={}",
                total,
                opts.max_total_bytes
            ));
        }

        let rf = RepoFile {
            path,
            size,
            sha256: None, // computed later by snapshot_from_files if bytes are included
            mode: f.mode.clone(),
            bytes: if opts.include_contents { f.bytes.clone() } else { None },
        };
        out.push(rf);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rejects_dotdot() {
        assert!(normalize_repo_path("../x").is_err());
        assert!(normalize_repo_path("a/../b").is_err());
    }

    #[test]
    fn include_exclude_rules() {
        let inc = vec!["src/**".to_string()];
        let exc = vec!["**/test*".to_string()];

        assert!(is_included("src/lib.rs", &inc, &exc));
        assert!(!is_included("src/test.rs", &inc, &exc));
        assert!(!is_included("README.md", &inc, &exc));
    }

    #[test]
    fn walk_is_deterministic_sorted() {
        let files = vec![
            VFile::new("b.txt", 1),
            VFile::new("a.txt", 1),
            VFile::new("./c.txt", 1),
        ];

        let out = walk_virtual_files(&files, &WalkOptions::default()).unwrap();
        let paths: Vec<String> = out.into_iter().map(|f| f.path).collect();
        assert_eq!(paths, vec!["a.txt", "b.txt", "c.txt"]);
    }
}
