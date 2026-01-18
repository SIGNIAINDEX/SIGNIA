//! Stable sorting utilities for SIGNIA.
//!
//! This module provides explicit, deterministic sorting helpers that:
//! - avoid platform-dependent ordering
//! - avoid HashMap iteration
//! - make ordering rules visible and auditable
//!
//! These helpers are intentionally verbose to make determinism guarantees clear.

use crate::errors::{SigniaError, SigniaResult};

/// Sort a vector by a key extractor in a stable, deterministic way.
///
/// This function is a thin wrapper around `sort_by`, but enforces:
/// - total ordering
/// - no NaN or incomparable values
pub fn stable_sort_by_key<T, K, F>(items: &mut Vec<T>, mut key_fn: F) -> SigniaResult<()>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    // Rust's slice::sort_by is stable.
    items.sort_by(|a, b| {
        let ka = key_fn(a);
        let kb = key_fn(b);
        ka.cmp(&kb)
    });
    Ok(())
}

/// Sort a vector of strings lexicographically.
///
/// Explicit helper to avoid ad-hoc ordering.
pub fn stable_sort_strings(items: &mut Vec<String>) {
    items.sort();
}

/// Sort a vector of (K, V) pairs by key.
///
/// Useful when working with decoded but unordered structures.
pub fn stable_sort_pairs<K, V>(items: &mut Vec<(K, V)>)
where
    K: Ord,
{
    items.sort_by(|a, b| a.0.cmp(&b.0));
}

/// Ensure a vector is already sorted.
///
/// Returns an error if the vector is not sorted.
pub fn ensure_sorted<T, K, F>(items: &[T], mut key_fn: F) -> SigniaResult<()>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    for w in items.windows(2) {
        let a = key_fn(&w[0]);
        let b = key_fn(&w[1]);
        if a > b {
            return Err(SigniaError::invariant(
                "collection is not sorted deterministically",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_sort_basic() {
        let mut v = vec![3, 1, 2];
        stable_sort_by_key(&mut v, |x| *x).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn ensure_sorted_detects_unsorted() {
        let v = vec![1, 3, 2];
        let err = ensure_sorted(&v, |x| *x).err().unwrap();
        assert!(err.to_string().contains("not sorted"));
    }
}
