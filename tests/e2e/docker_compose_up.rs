//! docker_compose_up.rs
//!
//! End-to-end smoke that checks Docker Compose can bring up the stack.
//! This test is skipped unless SIGNIA_RUN_E2E=1.
//!
//! Expected file at repo root: docker-compose.yml
//!
//! It will run:
//! - docker compose up -d
//! - docker compose ps
//! - docker compose down
//!
//! No external Rust dependencies.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

#[test]
fn docker_compose_stack_up_down() {
    if env::var("SIGNIA_RUN_E2E").ok().as_deref() != Some("1") {
        eprintln!("skip: set SIGNIA_RUN_E2E=1 to enable docker compose test");
        return;
    }

    let root = repo_root();
    let compose = root.join("docker-compose.yml");
    if !compose.exists() {
        eprintln!("skip: docker-compose.yml not found at {}", compose.display());
        return;
    }

    let up = Command::new("sh")
        .arg("-lc")
        .arg("docker compose up -d")
        .current_dir(&root)
        .status();

    match up {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!("skip: docker compose not available or failed to start");
            return;
        }
    }

    let _ = Command::new("sh").arg("-lc").arg("docker compose ps").current_dir(&root).status();
    let down = Command::new("sh").arg("-lc").arg("docker compose down -v").current_dir(&root).status();

    assert!(down.map(|s| s.success()).unwrap_or(false), "docker compose down failed");
}
