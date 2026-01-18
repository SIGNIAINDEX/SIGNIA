use anyhow::Result;

mod args;
mod cmd;
mod io;
mod output;
mod solana;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = args::Cli::parse();
    output::init(cli.json);

    cmd::dispatch(cli).await
}
