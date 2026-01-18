//! Constants shared between on-chain program and clients.
//!
//! Keep these stable because they affect PDA derivation.

use solana_program::pubkey::Pubkey;

/// PDA seed for the registry root account.
pub const SEED_REGISTRY: &[u8] = b"signia:registry";

/// PDA seed for namespace accounts.
pub const SEED_NAMESPACE: &[u8] = b"signia:namespace";

/// PDA seed for artifact records.
pub const SEED_RECORD: &[u8] = b"signia:record";

/// PDA seed for authority config.
pub const SEED_AUTH: &[u8] = b"signia:auth";

/// Default program id (placeholder).
///
/// Replace this with the deployed program id when available.
pub const DEFAULT_PROGRAM_ID: &str = "Signia1111111111111111111111111111111111111";

pub fn default_program_id() -> Pubkey {
    DEFAULT_PROGRAM_ID.parse().unwrap_or_else(|_| Pubkey::default())
}

/// Version string embedded into client metadata and instruction tags.
pub const CLIENT_VERSION: &str = "v1";
