//! Registry client for SIGNIA on-chain program.
//!
//! This client can:
//! - derive PDAs
//! - build instructions (create namespace, publish record)
//! - optionally submit transactions via RPC
//!
//! The actual on-chain program is expected to be implemented in `signia-program`.
//! This crate provides the off-chain wiring for UIs/CLI/servers.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use crate::pda;
use crate::constants::CLIENT_VERSION;

#[derive(Debug, Clone)]
pub struct RegistryClient {
    pub program_id: Pubkey,
    pub rpc: Option<RpcClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRecordArgs {
    pub namespace: String,
    pub object_id: String,
    /// Optional pointer to off-chain blob (e.g. HTTP URL, IPFS, Arweave).
    #[serde(default)]
    pub uri: Option<String>,
    /// Optional type hint (schema/manifest/proof).
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceArgs {
    pub namespace: String,
    /// Namespace authority (usually a wallet pubkey).
    pub authority: String,
}

impl RegistryClient {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id, rpc: None }
    }

    pub fn with_rpc(program_id: Pubkey, rpc_url: &str) -> Self {
        Self { program_id, rpc: Some(RpcClient::new(rpc_url.to_string())) }
    }

    pub fn derive_namespace(&self, namespace: &str) -> (Pubkey, u8) {
        pda::derive_namespace(&self.program_id, namespace)
    }

    pub fn derive_record(&self, namespace: &str, object_id: &str) -> (Pubkey, u8) {
        pda::derive_record(&self.program_id, namespace, object_id)
    }

    /// Build instruction to create a namespace account.
    pub fn ix_create_namespace(&self, payer: Pubkey, args: CreateNamespaceArgs) -> Result<Instruction> {
        let authority: Pubkey = args.authority.parse().map_err(|_| anyhow!("invalid authority pubkey"))?;
        let (ns_pda, ns_bump) = self.derive_namespace(&args.namespace);
        let (auth_pda, auth_bump) = pda::derive_namespace_auth(&self.program_id, &args.namespace);

        let data = RegistryIx::CreateNamespace {
            version: CLIENT_VERSION.to_string(),
            namespace: args.namespace,
            authority,
            ns_bump,
            auth_bump,
        }
        .to_vec()?;

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(ns_pda, false),
                AccountMeta::new(auth_pda, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data,
        })
    }

    /// Build instruction to publish a record within a namespace.
    pub fn ix_publish_record(&self, payer: Pubkey, authority: Pubkey, args: PublishRecordArgs) -> Result<Instruction> {
        let (ns_pda, _ns_bump) = self.derive_namespace(&args.namespace);
        let (auth_pda, auth_bump) = pda::derive_namespace_auth(&self.program_id, &args.namespace);
        let (record_pda, record_bump) = self.derive_record(&args.namespace, &args.object_id);

        let data = RegistryIx::PublishRecord {
            version: CLIENT_VERSION.to_string(),
            namespace: args.namespace,
            object_id: args.object_id,
            uri: args.uri,
            kind: args.kind,
            auth_bump,
            record_bump,
        }
        .to_vec()?;

        Ok(Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(payer, true),
                AccountMeta::new(authority, true),
                AccountMeta::new(ns_pda, false),
                AccountMeta::new(auth_pda, false),
                AccountMeta::new(record_pda, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data,
        })
    }

    /// Submit a transaction. Requires the client to be constructed with RPC.
    pub fn send_transaction(&self, payer: &Keypair, ixs: &[Instruction]) -> Result<String> {
        let rpc = self.rpc.as_ref().ok_or_else(|| anyhow!("rpc client not configured"))?;
        let bh = rpc.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), &[payer], bh);
        let sig = rpc.send_and_confirm_transaction(&tx)?;
        Ok(sig.to_string())
    }
}

/// Registry program instruction encoding.
///
/// This encoding is designed to be stable and easy to decode on-chain.
/// It uses a small tag byte followed by bincode-encoded payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RegistryIx {
    CreateNamespace {
        version: String,
        namespace: String,
        authority: Pubkey,
        ns_bump: u8,
        auth_bump: u8,
    },
    PublishRecord {
        version: String,
        namespace: String,
        object_id: String,
        uri: Option<String>,
        kind: Option<String>,
        auth_bump: u8,
        record_bump: u8,
    },
}

impl RegistryIx {
    fn to_vec(&self) -> Result<Vec<u8>> {
        // Tag: 1 byte
        let tag = match self {
            RegistryIx::CreateNamespace { .. } => 1u8,
            RegistryIx::PublishRecord { .. } => 2u8,
        };
        let mut out = vec![tag];
        let payload = bincode::serialize(self).map_err(|e| anyhow!("serialize: {e}"))?;
        out.extend_from_slice(&payload);
        Ok(out)
    }

    #[allow(dead_code)]
    fn from_slice(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow!("empty instruction data"));
        }
        let _tag = data[0];
        let ix: RegistryIx = bincode::deserialize(&data[1..]).map_err(|e| anyhow!("deserialize: {e}"))?;
        Ok(ix)
    }
}
