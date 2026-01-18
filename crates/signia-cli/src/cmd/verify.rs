use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::io::input;
use crate::output;

#[derive(Debug, Serialize)]
pub struct VerifyOut {
    pub ok: bool,
}

pub async fn run(root_hex: &str, leaf_hex: &str, proof_path: &str) -> Result<()> {
    let proof_json = input::read_json_file(proof_path)?;
    let proof: signia_store::proofs::merkle::MerkleProof = serde_json::from_value(proof_json)
        .map_err(|e| anyhow!("invalid proof json: {e}"))?;

    let root_bytes = hex::decode(root_hex).map_err(|_| anyhow!("root must be hex"))?;
    if root_bytes.len() != 32 {
        return Err(anyhow!("root must be 32 bytes"));
    }
    let mut root = [0u8; 32];
    root.copy_from_slice(&root_bytes);

    let ok = signia_store::proofs::verify::verify_proof(leaf_hex, &root, &proof)?;
    output::print(&VerifyOut { ok })?;
    Ok(())
}
