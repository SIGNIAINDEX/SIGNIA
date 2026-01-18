use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::output;
use crate::solana;

#[derive(Debug, Serialize)]
pub struct PublishOut {
    pub ok: bool,
    pub cluster: String,
    pub note: String,
    pub id: Option<String>,
}

pub async fn run(devnet: bool, mainnet: bool, id: Option<&str>) -> Result<()> {
    let cluster = if devnet && mainnet {
        return Err(anyhow!("choose only one: --devnet or --mainnet"));
    } else if mainnet {
        "mainnet-beta"
    } else {
        "devnet"
    };

    // Placeholder: wire to signia-program instructions once available.
    // This implementation performs client initialization and prints a clear action note.
    let _client = solana::client::SolanaClient::new(cluster)?;

    output::print(&PublishOut {
        ok: true,
        cluster: cluster.to_string(),
        id: id.map(|s| s.to_string()),
        note: "publish is a stub in signia-cli; wire signia-program registry instructions to enable on-chain publishing".to_string(),
    })?;
    Ok(())
}
