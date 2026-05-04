//! TorrentsTrackers command-line interface.
//!
//! Phase 1 MVP: manage sources & pools, sync them, search across the local
//! catalog, open magnets in the system torrent client.

mod commands;
mod fmt;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "tt",
    version,
    about = "TorrentsTrackers — federated torrent discovery (CLI)",
    long_about = None,
)]
struct Cli {
    /// Override the SQLite database path.
    #[arg(long, global = true)]
    db: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage sources (communities the app pulls from).
    #[command(subcommand)]
    Source(commands::source::SourceCmd),

    /// Manage pools (user-defined combinations of sources).
    #[command(subcommand)]
    Pool(commands::pool::PoolCmd),

    /// Search across the local catalog.
    Search(commands::search::SearchArgs),

    /// Open an entry's magnet in the system torrent client.
    Open(commands::open::OpenArgs),

    /// Show app paths and database stats.
    Info,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let db_path = match cli.db {
        Some(p) => p,
        None => tt_storage::paths::db_path()?,
    };
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = tt_storage::Database::open(&db_path)?;

    match cli.command {
        Command::Source(cmd) => commands::source::run(cmd, &db).await?,
        Command::Pool(cmd) => commands::pool::run(cmd, &db)?,
        Command::Search(args) => commands::search::run(args, &db)?,
        Command::Open(args) => commands::open::run(args, &db)?,
        Command::Info => commands::info::run(&db_path, &db)?,
    }

    Ok(())
}
