use anyhow::Result;

use crate::args::{Cli, Command};

mod compile;
mod doctor;
mod fetch;
mod plugins;
mod publish;
mod verify;

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Compile { input, kind, out } => compile::run(&cli.store_root, &input, kind.as_deref(), &out).await,
        Command::Verify { root, leaf, proof } => verify::run(&root, &leaf, &proof).await,
        Command::Fetch { id, to } => fetch::run(&cli.store_root, &id, to.as_deref()).await,
        Command::Plugins => plugins::run(&cli.store_root).await,
        Command::Doctor => doctor::run().await,
        Command::Publish { devnet, mainnet, id } => publish::run(devnet, mainnet, id.as_deref()).await,
    }
}
