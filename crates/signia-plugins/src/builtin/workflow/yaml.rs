//! YAML support for the built-in `workflow` plugin.
//!
//! This module provides deterministic parsing and normalization from YAML into the
//! canonical JSON shape expected by `builtin.workflow`.
//!
//! Why YAML:
//! - workflows are often authored as YAML in CI/CD and orchestration systems
//! - YAML is human-friendly for quick iteration
//!
//! Determinism rules:
//! - YAML is parsed into a serde_yaml::Value, then converted into serde_json::Value
//! - mapping keys are normalized to stable strings
//! - the resulting JSON is canonicalized (sorted keys) before hashing/usage
//!
//! I/O rules:
//! - no filesystem/network I/O
//! - caller provides YAML text/bytes
//!
//! Feature gates:
//! - requires the `yaml` Cargo feature for `serde_yaml` dependency

#![cfg(all(feature = "builtin", feature = "yaml"))]

use anyhow::{anyhow, Result};
use serde_json::Value;

use signia_core::determinism::canonical_json::canonicalize_json;

/// Parse workflow YAML to the canonical JSON shape used by SIGNIA.
///
/// Expected YAML structure (example):
///
/// ```yaml
/// name: demo
/// version: v1
/// nodes:
///   - id: a
///     type: http
///     meta:
///       url: https://example.com
///   - id: b
///     type: llm
/// edges:
///   - from: a
///     to: b
///     kind: data
///     label: response
/// ```
pub fn parse_workflow_yaml(yaml_text: &str) -> Result<Value> {
    if yaml_text.trim().is_empty() {
        return Err(anyhow!("workflow yaml is empty"));
    }

    let y: serde_yaml::Value = serde_yaml::from_str(yaml_text)
        .map_err(|e| anyhow!("failed to parse yaml: {e}"))?;

    let j = yaml_to_json(&y)?;
    let c = canonicalize_json(&j)?;
    Ok(c)
}

/// Convert YAML value to JSON deterministically.
pub fn yaml_to_json(v: &serde_yaml::Value) -> Result<Value> {
    match v {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_yaml::Value::Number(n) => {
            // serde_yaml numbers can be i64/f64/u64
            if let Some(i) = n.as_i64() {
                Ok(Value::Number(i.into()))
            } else if let Some(u) = n.as_u64() {
                Ok(Value::Number(serde_json::Number::from(u)))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| anyhow!("invalid float in yaml"))
            } else {
                Err(anyhow!("unknown numeric type in yaml"))
            }
        }
        serde_yaml::Value::String(s) => Ok(Value::String(s.clone())),
        serde_yaml::Value::Sequence(seq) => {
            let mut out = Vec::with_capacity(seq.len());
            for item in seq {
                out.push(yaml_to_json(item)?);
            }
            Ok(Value::Array(out))
        }
        serde_yaml::Value::Mapping(map) => {
            // YAML keys can be complex; we normalize keys to strings deterministically.
            // Strategy:
            // - if key is String => use it
            // - otherwise serialize key to YAML and use that as stable string
            let mut pairs: Vec<(String, Value)> = Vec::with_capacity(map.len());
            for (k, v2) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => {
                        // Deterministic string for non-string keys
                        // serde_yaml::to_string is stable for a single value.
                        let ks = serde_yaml::to_string(k).unwrap_or_else(|_| "<key>".to_string());
                        ks.trim().to_string()
                    }
                };
                pairs.push((key, yaml_to_json(v2)?));
            }

            // Deterministic ordering by key
            pairs.sort_by(|a, b| a.0.cmp(&b.0));

            let mut obj = serde_json::Map::new();
            for (k, v2) in pairs {
                obj.insert(k, v2);
            }
            Ok(Value::Object(obj))
        }
        // Tagged/other variants (serde_yaml may represent as Mapping/Sequence/String already)
        _ => Err(anyhow!("unsupported yaml value kind")),
    }
}

/// Validate that the parsed workflow JSON matches the minimal required shape.
/// This is a lightweight guard; the plugin will do strict validation as well.
pub fn validate_workflow_json(j: &Value) -> Result<()> {
    let obj = j.as_object().ok_or_else(|| anyhow!("workflow must be a JSON object"))?;

    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if name.trim().is_empty() {
        return Err(anyhow!("workflow.name is required"));
    }

    let nodes = obj
        .get("nodes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("workflow.nodes must be an array"))?;
    if nodes.is_empty() {
        return Err(anyhow!("workflow.nodes must not be empty"));
    }

    // Basic node fields
    for n in nodes {
        let no = n.as_object().ok_or_else(|| anyhow!("workflow node must be an object"))?;
        let id = no.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let ty = no.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if id.is_empty() || ty.is_empty() {
            return Err(anyhow!("workflow node requires id and type"));
        }
    }

    // Edges optional but if present must be array
    if let Some(e) = obj.get("edges") {
        if !e.is_array() {
            return Err(anyhow!("workflow.edges must be an array"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_validate_yaml() {
        let y = r#"
name: demo
version: v1
nodes:
  - id: a
    type: http
    meta:
      url: https://example.com
  - id: b
    type: llm
edges:
  - from: a
    to: b
    kind: data
    label: response
"#;
        let j = parse_workflow_yaml(y).unwrap();
        validate_workflow_json(&j).unwrap();
        assert!(j.get("name").is_some());
        assert!(j.get("nodes").is_some());
    }

    #[test]
    fn yaml_mapping_key_sort_is_stable() {
        let y = r#"
name: demo
nodes:
  - id: b
    type: t
  - id: a
    type: t
edges: []
"#;
        let j = parse_workflow_yaml(y).unwrap();
        let s1 = serde_json::to_string(&j).unwrap();
        let s2 = serde_json::to_string(&parse_workflow_yaml(y).unwrap()).unwrap();
        assert_eq!(s1, s2);
    }
}
