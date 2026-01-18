use anyhow::Result;
use serde::Serialize;

use crate::output;

#[derive(Debug, Serialize)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct DoctorOut {
    pub ok: bool,
    pub checks: Vec<Check>,
}

pub async fn run() -> Result<()> {
    let mut checks = Vec::new();

    // Basic: rust version
    checks.push(Check {
        name: "rustc".to_string(),
        ok: which_ok("rustc"),
        detail: "required for building".to_string(),
    });

    checks.push(Check {
        name: "cargo".to_string(),
        ok: which_ok("cargo"),
        detail: "required for building".to_string(),
    });

    // Solana tooling is optional but recommended.
    checks.push(Check {
        name: "solana".to_string(),
        ok: which_ok("solana"),
        detail: "optional (required for publish to on-chain registry)".to_string(),
    });

    let ok = checks.iter().all(|c| c.ok || c.name == "solana");
    output::print(&DoctorOut { ok, checks })?;
    Ok(())
}

fn which_ok(cmd: &str) -> bool {
    std::env::var_os("PATH").and_then(|paths| {
        for p in std::env::split_paths(&paths) {
            let full = p.join(cmd);
            if full.exists() {
                return Some(());
            }
            #[cfg(windows)]
            {
                let full_exe = p.join(format!("{cmd}.exe"));
                if full_exe.exists() {
                    return Some(());
                }
            }
        }
        None
    }).is_some()
}
