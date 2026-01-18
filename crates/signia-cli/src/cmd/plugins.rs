use anyhow::Result;
use serde::Serialize;

use crate::output;

#[derive(Debug, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub version: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct PluginsOut {
    pub plugins: Vec<PluginInfo>,
}

pub async fn run(_store_root: &str) -> Result<()> {
    let mut reg = signia_plugins::registry::PluginRegistry::default();
    signia_plugins::builtin::repo::register(&mut reg);
    signia_plugins::builtin::dataset::register(&mut reg);
    signia_plugins::builtin::workflow::register(&mut reg);
    signia_plugins::builtin::api::register(&mut reg);
    signia_plugins::builtin::spec::register(&mut reg);

    let plugins = reg
        .list()
        .into_iter()
        .map(|s| PluginInfo { id: s.id, version: s.version, kind: s.kind })
        .collect();

    output::print(&PluginsOut { plugins })?;
    Ok(())
}
