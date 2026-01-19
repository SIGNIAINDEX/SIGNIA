//! schema_validation.rs
//!
//! Validates that CLI output contains required top-level fields.
//! This is a pragmatic validation that does not require a JSON Schema engine.
//!
//! A stricter JSON Schema validation can be added later if you decide to ship
//! a dedicated test harness crate with `jsonschema`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn signia_bin() -> Option<PathBuf> {
    if let Ok(p) = env::var("SIGNIA_BIN") {
        let pb = PathBuf::from(p);
        if pb.exists() { return Some(pb); }
    }
    let p = repo_root().join("target").join("debug").join(if cfg!(windows) { "signia.exe" } else { "signia" });
    if p.exists() { Some(p) } else { None }
}

fn run_compile(bin: &Path, input: &Path, typ: &str, out: &Path) {
    let status = Command::new(bin)
        .arg("compile")
        .arg(input)
        .arg("--type").arg(typ)
        .arg("--out").arg(out)
        .status()
        .expect("failed to spawn signia");
    assert!(status.success(), "signia compile failed");
}

fn json_contains_all(s: &str, keys: &[&str]) -> bool {
    keys.iter().all(|k| s.contains(&format!("\"{k}\"")))
}

#[test]
fn schema_has_required_fields() {
    let Some(bin) = signia_bin() else {
        eprintln!("skip: signia CLI not found (set SIGNIA_BIN or build signia-cli)");
        return;
    };

    let root = repo_root();
    let input = root.join("tests").join("fixtures").join("openapi_petstore").join("petstore.yaml");

    let out = root.join("target").join("tmp").join("signia_test_schema_validation");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    run_compile(&bin, &input, "openapi", &out);

    let schema = fs::read_to_string(out.join("schema.json")).expect("schema.json missing");
    let manifest = fs::read_to_string(out.join("manifest.json")).expect("manifest.json missing");
    let proof = fs::read_to_string(out.join("proof.json")).expect("proof.json missing");

    assert!(json_contains_all(&schema, &["version", "kind", "nodes", "edges"]), "schema.json missing required keys");
    assert!(json_contains_all(&manifest, &["schemaHash", "artifactHashes"]), "manifest.json missing required keys");
    assert!(json_contains_all(&proof, &["root", "leaves"]), "proof.json missing required keys");
}
