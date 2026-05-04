//! TorrentsTrackers command-line interface (`tt` binary).
//!
//! Phase 1 will wire up actual commands. For now, the binary just prints
//! a banner and exits — enough to keep the workspace compiling.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tt", version, about = "TorrentsTrackers — federated torrent discovery")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Manage sources (communities) the app pulls from.
    Source,
    /// Manage pools (combinations of sources).
    Pool,
    /// Search for entries across configured sources.
    Search,
    /// Manage local cryptographic identity.
    Identity,
    /// Open an entry's magnet in the configured torrent client.
    Open,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("TorrentsTrackers CLI — Phase 0 skeleton.");
            println!("Run `tt --help` for available commands.");
        }
        Some(_) => {
            println!("Command not implemented yet — see ROADMAP.md (Phase 1).");
        }
    }

    Ok(())
}
