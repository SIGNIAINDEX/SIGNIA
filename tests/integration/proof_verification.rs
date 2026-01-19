//! proof_verification.rs
//!
//! Uses the CLI `verify` command to validate schema/proof/manifest linkage.

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

#[test]
fn verify_bundle_proof() {
    let Some(bin) = signia_bin() else {
        eprintln!("skip: signia CLI not found (set SIGNIA_BIN or build signia-cli)");
        return;
    };

    let root = repo_root();
    let input = root.join("tests").join("fixtures").join("workflow_small").join("pipeline.yml");

    let out = root.join("target").join("tmp").join("signia_test_proof_verification");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    run_compile(&bin, &input, "workflow", &out);

    let status = Command::new(&bin)
        .arg("verify")
        .arg(out.join("schema.json"))
        .arg(out.join("proof.json"))
        .arg("--manifest")
        .arg(out.join("manifest.json"))
        .status()
        .expect("failed to run signia verify");

    assert!(status.success(), "signia verify failed");
}
