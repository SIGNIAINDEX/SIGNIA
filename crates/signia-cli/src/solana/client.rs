use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct SolanaClient {
    pub cluster: String,
}

impl SolanaClient {
    pub fn new(cluster: &str) -> Result<Self> {
        if cluster.trim().is_empty() {
            return Err(anyhow!("cluster must not be empty"));
        }
        Ok(Self { cluster: cluster.to_string() })
    }
}
