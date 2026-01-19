//! determinism_roundtrip.rs
//!
//! Black-box determinism test:
//! same input => same output byte-for-byte.
//!
//! This test executes the `signia` CLI twice and compares produced bundles.
//!
//! How to run:
//! - build CLI: `cargo build -p signia-cli`
//! - then: `cargo test -q` (from workspace root)
//!
//! Notes:
//! - The CLI path can be overridden via SIGNIA_BIN.
//! - If the CLI binary is not found, the test is skipped.

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

fn read_bytes(p: &Path) -> Vec<u8> {
    fs::read(p).unwrap_or_else(|e| panic!("failed to read {}: {e}", p.display()))
}

#[test]
fn determinism_dataset() {
    let Some(bin) = signia_bin() else {
        eprintln!("skip: signia CLI not found (set SIGNIA_BIN or build signia-cli)");
        return;
    };

    let root = repo_root();
    let input = root.join("tests").join("fixtures").join("dataset_small").join("sample.csv");

    let out1 = root.join("target").join("tmp").join("signia_test_determinism_1");
    let out2 = root.join("target").join("tmp").join("signia_test_determinism_2");
    let _ = fs::remove_dir_all(&out1);
    let _ = fs::remove_dir_all(&out2);
    fs::create_dir_all(&out1).unwrap();
    fs::create_dir_all(&out2).unwrap();

    run_compile(&bin, &input, "dataset", &out1);
    run_compile(&bin, &input, "dataset", &out2);

    for name in ["schema.json", "manifest.json", "proof.json"] {
        let b1 = read_bytes(&out1.join(name));
        let b2 = read_bytes(&out2.join(name));
        assert_eq!(b1, b2, "bundle file differs: {name}");
    }
}
