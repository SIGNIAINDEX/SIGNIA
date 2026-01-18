//! Text normalization utilities for SIGNIA.
//!
//! This module defines deterministic text normalization rules applied before
//! hashing, comparison, or schema emission.
//!
//! Goals:
//! - identical logical text yields identical normalized output
//! - avoid platform-specific newline and encoding differences
//! - remain purely in-memory (no I/O)
//!
//! These utilities are intentionally conservative and explicit.

use crate::errors::{SigniaError, SigniaResult};

/// Normalize text content deterministically.
///
/// Rules:
/// - convert CRLF and CR to LF
/// - remove UTF-8 BOM if present
/// - trim trailing whitespace on each line
/// - ensure final newline is preserved if originally present
pub fn normalize_text(input: &str) -> SigniaResult<String> {
    if input.is_empty() {
        return Ok(String::new());
    }

    let mut s = input.to_string();

    // Remove UTF-8 BOM if present
    if s.starts_with('\u{FEFF}') {
        s = s.trim_start_matches('\u{FEFF}').to_string();
    }

    // Normalize newlines to LF
    s = s.replace("\r\n", "\n").replace('\r', "\n");

    let had_trailing_newline = s.ends_with('\n');

    // Trim trailing whitespace per line
    let mut lines: Vec<String> = Vec::new();
    for line in s.split('\n') {
        lines.push(line.trim_end().to_string());
    }

    let mut out = lines.join("\n");

    if had_trailing_newline {
        out.push('\n');
    }

    Ok(out)
}

/// Normalize text and enforce a maximum byte size after normalization.
pub fn normalize_text_with_limit(input: &str, max_bytes: usize) -> SigniaResult<String> {
    let out = normalize_text(input)?;
    if out.as_bytes().len() > max_bytes {
        return Err(SigniaError::invalid_argument(
            "normalized text exceeds maximum size",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_newlines_and_whitespace() {
        let s = "a \r\nb\t\r\n";
        let n = normalize_text(s).unwrap();
        assert_eq!(n, "a\nb\n");
    }

    #[test]
    fn remove_bom() {
        let s = "\u{FEFF}hello\n";
        let n = normalize_text(s).unwrap();
        assert_eq!(n, "hello\n");
    }

    #[test]
    fn size_limit_enforced() {
        let s = "a";
        let err = normalize_text_with_limit(s, 0).err().unwrap();
        assert!(err.to_string().contains("exceeds"));
    }
}
