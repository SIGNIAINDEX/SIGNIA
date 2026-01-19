//! api_compile_flow.rs
//!
//! Starts `signia-api` as a subprocess and calls /v1/compile.
//!
//! This is an optional integration test and will be skipped unless:
//! - the `signia-api` binary exists (or SIGNIA_API_BIN is set)
//! - the selected port is free
//!
//! Environment:
//! - SIGNIA_API_BIN: path to the signia-api binary
//! - SIGNIA_API_PORT: port to bind (default 8787)

use std::env;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn api_bin() -> Option<PathBuf> {
    if let Ok(p) = env::var("SIGNIA_API_BIN") {
        let pb = PathBuf::from(p);
        if pb.exists() { return Some(pb); }
    }
    let p = repo_root().join("target").join("debug").join(if cfg!(windows) { "signia-api.exe" } else { "signia-api" });
    if p.exists() { Some(p) } else { None }
}

fn can_bind(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[test]
fn api_compile_smoke() {
    let Some(bin) = api_bin() else {
        eprintln!("skip: signia-api binary not found (set SIGNIA_API_BIN or build signia-api)");
        return;
    };

    let port: u16 = env::var("SIGNIA_API_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8787);
    if !can_bind(port) {
        eprintln!("skip: port {} is not available", port);
        return;
    }

    let mut child = Command::new(&bin)
        .env("SIGNIA_BIND_ADDR", format!("127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start signia-api");

    // give it time to boot
    thread::sleep(Duration::from_millis(700));

    // Use curl if available (no external Rust deps).
    let payload = r#"{"type":"dataset","content":"id,name\n1,a\n"}"#;

    let status = Command::new("sh")
        .arg("-lc")
        .arg(format!(
            "curl -fsS -H 'content-type: application/json' -d '{}' http://127.0.0.1:{}/v1/compile > /dev/null",
            payload.replace("'", "'\"'\"'"),
            port
        ))
        .status();

    // best effort cleanup
    let _ = child.kill();

    match status {
        Ok(s) => assert!(s.success(), "curl request to /v1/compile failed"),
        Err(_) => {
            eprintln!("skip: curl is not available in this environment");
        }
    }
}
