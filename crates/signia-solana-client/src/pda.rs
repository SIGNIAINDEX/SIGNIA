//! PDA derivation helpers for the SIGNIA registry program.
//!
//! These helpers implement deterministic address derivation and are designed to
//! match the on-chain program's seeds and layout.

use solana_program::pubkey::Pubkey;

use crate::constants::{SEED_AUTH, SEED_NAMESPACE, SEED_RECORD, SEED_REGISTRY};

#[derive(Debug, Clone)]
pub struct RegistryPdas {
    pub registry: (Pubkey, u8),
}

#[derive(Debug, Clone)]
pub struct NamespacePdas {
    pub namespace: (Pubkey, u8),
    pub auth: (Pubkey, u8),
}

#[derive(Debug, Clone)]
pub struct RecordPdas {
    pub record: (Pubkey, u8),
}

/// Derive the registry root PDA.
pub fn derive_registry(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SEED_REGISTRY], program_id)
}

/// Derive a namespace PDA by name.
///
/// Namespace names are normalized in a deterministic way by the client.
/// The canonical form is lowercase ASCII with '-' separators.
pub fn derive_namespace(program_id: &Pubkey, namespace: &str) -> (Pubkey, u8) {
    let ns = normalize_namespace(namespace);
    Pubkey::find_program_address(&[SEED_NAMESPACE, ns.as_bytes()], program_id)
}

/// Derive the namespace authority PDA.
pub fn derive_namespace_auth(program_id: &Pubkey, namespace: &str) -> (Pubkey, u8) {
    let ns = normalize_namespace(namespace);
    Pubkey::find_program_address(&[SEED_AUTH, ns.as_bytes()], program_id)
}

/// Derive a record PDA by namespace + object id.
///
/// Object id should be a stable content-addressed id (e.g. sha256 hex).
pub fn derive_record(program_id: &Pubkey, namespace: &str, object_id: &str) -> (Pubkey, u8) {
    let ns = normalize_namespace(namespace);
    let oid = normalize_object_id(object_id);
    Pubkey::find_program_address(&[SEED_RECORD, ns.as_bytes(), oid.as_bytes()], program_id)
}

/// Collect PDAs used by most flows.
pub fn pdas_for_namespace(program_id: &Pubkey, namespace: &str) -> NamespacePdas {
    NamespacePdas {
        namespace: derive_namespace(program_id, namespace),
        auth: derive_namespace_auth(program_id, namespace),
    }
}

pub fn pdas_for_record(program_id: &Pubkey, namespace: &str, object_id: &str) -> RecordPdas {
    RecordPdas { record: derive_record(program_id, namespace, object_id) }
}

fn normalize_namespace(input: &str) -> String {
    let mut out = String::new();
    for c in input.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c == '-' || c == '_' || c == ' ' || c == '.' {
            if !out.ends_with('-') && !out.is_empty() {
                out.push('-');
            }
        }
    }
    out.trim_matches('-').to_string()
}

fn normalize_object_id(input: &str) -> String {
    // Accept sha256 hex or base58; normalize to lowercase hex if possible.
    let s = input.trim();
    if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return s.to_ascii_lowercase();
    }
    // If base58, convert to bytes and hex encode.
    if let Ok(bytes) = bs58::decode(s).into_vec() {
        return hex::encode(bytes);
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_normalization_is_stable() {
        assert_eq!(normalize_namespace("My Space"), "my-space");
        assert_eq!(normalize_namespace("my_space"), "my-space");
        assert_eq!(normalize_namespace("my..space"), "my-space");
        assert_eq!(normalize_namespace("  my-space  "), "my-space");
    }

    #[test]
    fn object_id_normalization_hex() {
        let h = "A".repeat(64);
        assert_eq!(normalize_object_id(&h), "a".repeat(64));
    }
}
