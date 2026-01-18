use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(name = "signia", version, about = "SIGNIA CLI")]
pub struct Cli {
    /// Emit JSON output on stdout.
    #[arg(long, global = true)]
    pub json: bool,

    /// Store root directory (default: .signia)
    #[arg(long, global = true, default_value = ".signia")]
    pub store_root: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Compile a structure input into deterministic artifacts.
    Compile {
        /// Input path or URL.
        input: String,

        /// Optional kind hint: repo|dataset|workflow|openapi
        #[arg(long)]
        kind: Option<String>,

        /// Output directory to write schema/manifest/proof.
        #[arg(long, default_value = "./out")]
        out: String,
    },

    /// Verify a Merkle inclusion proof.
    Verify {
        #[arg(long)]
        root: String,
        #[arg(long)]
        leaf: String,
        /// Proof JSON file (MerkleProof structure).
        #[arg(long)]
        proof: String,
    },

    /// Fetch an artifact from the local store by object id.
    Fetch {
        id: String,
        #[arg(long)]
        to: Option<String>,
    },

    /// List available plugins and versions.
    Plugins,

    /// Run environment checks.
    Doctor,

    /// Publish compiled artifacts to an on-chain registry (placeholder).
    Publish {
        #[arg(long)]
        devnet: bool,
        #[arg(long)]
        mainnet: bool,
        /// Optional object id to publish (manifest or schema).
        #[arg(long)]
        id: Option<String>,
    },
}
