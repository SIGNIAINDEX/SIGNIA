use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use url::Url;

pub async fn resolve_to_json(input: &str) -> Result<serde_json::Value> {
    // 1) URL
    if looks_like_url(input) {
        return fetch_url_json(input).await;
    }

    // 2) GitHub shorthand: owner/repo[@ref][:path]
    if is_github_shorthand(input) {
        return fetch_github_shorthand_json(input).await;
    }

    // 3) Local file
    read_json_file(input)
}

pub fn read_json_file<P: AsRef<Path>>(path: P) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path.as_ref())?;
    let v: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| anyhow!("invalid json: {e}"))?;
    Ok(v)
}

async fn fetch_url_json(url: &str) -> Result<serde_json::Value> {
    let resp = reqwest::get(url).await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("http error: {status}"));
    }
    let v = resp.json::<serde_json::Value>().await?;
    Ok(v)
}

/// GitHub shorthand resolves to raw.githubusercontent.com.
/// Format: owner/repo[@ref][:path]
/// If no path is provided, defaults to `signia.json`.
async fn fetch_github_shorthand_json(s: &str) -> Result<serde_json::Value> {
    let (repo, ref_opt, path_opt) = parse_github_shorthand(s)?;
    let path = path_opt.unwrap_or_else(|| "signia.json".to_string());
    let r = ref_opt.unwrap_or_else(|| "main".to_string());

    let url = format!("https://raw.githubusercontent.com/{repo}/{r}/{path}");
    fetch_url_json(&url).await
}

fn looks_like_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}

fn is_github_shorthand(s: &str) -> bool {
    // Very lightweight test: contains exactly one '/'
    let parts: Vec<&str> = s.split('/').collect();
    parts.len() == 2 && parts[0].len() >= 1 && parts[1].len() >= 1
}

fn parse_github_shorthand(s: &str) -> Result<(String, Option<String>, Option<String>)> {
    // owner/repo[@ref][:path]
    let mut repo_part = s.to_string();
    let mut ref_part: Option<String> = None;
    let mut path_part: Option<String> = None;

    if let Some(idx) = repo_part.find(':') {
        path_part = Some(repo_part[idx + 1..].to_string());
        repo_part = repo_part[..idx].to_string();
    }
    if let Some(idx) = repo_part.find('@') {
        ref_part = Some(repo_part[idx + 1..].to_string());
        repo_part = repo_part[..idx].to_string();
    }

    if repo_part.split('/').count() != 2 {
        return Err(anyhow!("invalid github shorthand"));
    }
    Ok((repo_part, ref_part, path_part))
}
