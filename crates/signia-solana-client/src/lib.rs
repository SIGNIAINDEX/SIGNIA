//! signia-solana-client
//!
//! This crate provides a small, focused Rust client for interacting with the
//! SIGNIA on-chain registry program.
//!
//! It includes:
//! - PDA derivation helpers
//! - constant seeds and default program id placeholder
//! - a registry client that can build instructions and submit transactions
//!
//! Note: The on-chain program id is expected to be provided by the consumer.
//! The default here is a placeholder constant for local development.

pub mod constants;
pub mod pda;
pub mod registry_client;

pub use constants::*;
pub use pda::*;
pub use registry_client::*;
