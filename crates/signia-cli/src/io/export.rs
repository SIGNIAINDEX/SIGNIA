use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

pub fn write_bundle<P: AsRef<Path>>(out_dir: P, schema: &serde_json::Value, manifest: &serde_json::Value, proof: &serde_json::Value) -> Result<()> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    fs::write(out_dir.join("schema.json"), serde_json::to_vec_pretty(schema)?)?;
    fs::write(out_dir.join("manifest.json"), serde_json::to_vec_pretty(manifest)?)?;
    fs::write(out_dir.join("proof.json"), serde_json::to_vec_pretty(proof)?)?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub fn build_manifest(input: &serde_json::Value, schema_id: &str, kind: &str) -> serde_json::Value {
    let input_bytes = serde_json::to_vec(input).unwrap_or_default();
    serde_json::json!({
        "version": "v1",
        "inputKind": kind,
        "inputHash": sha256_hex(&input_bytes),
        "schemaObjectId": schema_id,
        "createdAt": time::OffsetDateTime::now_utc().unix_timestamp(),
    })
}

pub fn build_proof(input: &serde_json::Value, schema_id: &str, manifest_id: &str) -> Result<serde_json::Value> {
    let input_bytes = serde_json::to_vec(input)?;
    let leaf = sha256_hex(&input_bytes);
    let schema_leaf = sha256_hex(schema_id.as_bytes());

    let leaves = vec![leaf.clone(), schema_leaf.clone()];
    let root = signia_store::proofs::merkle::merkle_root_hex(&leaves)?;
    let proof0 = signia_store::proofs::merkle::merkle_proof(&leaves, 0).ok();

    Ok(serde_json::json!({
        "version": "v1",
        "root": root,
        "leaf": leaf,
        "schemaLeaf": schema_leaf,
        "manifestObjectId": manifest_id,
        "merkleProof": proof0,
    }))
}
