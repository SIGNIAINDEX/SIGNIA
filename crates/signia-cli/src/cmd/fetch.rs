use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::output;

#[derive(Debug, Serialize)]
pub struct FetchOut {
    pub id: String,
    pub bytes: usize,
    pub wrote_to: Option<String>,
}

pub async fn run(store_root: &str, id: &str, to: Option<&str>) -> Result<()> {
    let store_cfg = signia_store::StoreConfig::local_dev(PathBuf::from(store_root))?;
    let store = signia_store::Store::open(store_cfg)?;

    let Some(bytes) = store.get_object_bytes(id)? else {
        return Err(anyhow!("object not found"));
    };

    if let Some(path) = to {
        fs::write(path, &bytes)?;
        output::print(&FetchOut { id: id.to_string(), bytes: bytes.len(), wrote_to: Some(path.to_string()) })?;
    } else {
        // Print as base64-like hex preview only
        let preview = hex::encode(&bytes[..bytes.len().min(64)]);
        output::print(&FetchOut { id: id.to_string(), bytes: bytes.len(), wrote_to: None })?;
        if !output::is_json() {
            println!("preview_hex_64: {preview}");
        }
    }
    Ok(())
}
