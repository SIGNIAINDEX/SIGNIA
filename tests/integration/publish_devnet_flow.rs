//! publish_devnet_flow.rs
//!
//! Optional devnet publish flow test.
//!
//! This test is skipped by default. To enable, set:
//! - SIGNIA_RUN_DEVNET_TESTS=1
//! - SOLANA_URL (optional): defaults to https://api.devnet.solana.com
//! - SIGNIA_PUBLISH_DEVNET=1
//!
//! The test runs `signia publish --devnet` on a compiled schema bundle.

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
fn publish_devnet_smoke() {
    if env::var("SIGNIA_RUN_DEVNET_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skip: set SIGNIA_RUN_DEVNET_TESTS=1 to enable devnet flow test");
        return;
    }
    let Some(bin) = signia_bin() else {
        eprintln!("skip: signia CLI not found (set SIGNIA_BIN or build signia-cli)");
        return;
    };

    let root = repo_root();
    let input = root.join("tests").join("fixtures").join("openapi_petstore").join("petstore.yaml");

    let out = root.join("target").join("tmp").join("signia_test_publish_devnet");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    run_compile(&bin, &input, "openapi", &out);

    let status = Command::new(&bin)
        .arg("publish")
        .arg(out.join("schema.json"))
        .arg("--devnet")
        .status()
        .expect("failed to run signia publish");

    assert!(status.success(), "signia publish --devnet failed");
}
