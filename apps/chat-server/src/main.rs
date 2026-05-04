//! `tt-chat-server` — standalone WebSocket chat server.
//!
//! Designed to be run by the modos of a community: drop the binary on a
//! VPS, write a 5-line `tt-chat-server.toml`, port-forward, share the
//! `ws://your.host:6970/` URL with members.

mod config;
mod db;
mod handler;
mod state;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use axum::routing::get;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::state::AppState;

#[derive(Parser)]
#[command(
    name = "tt-chat-server",
    version,
    about = "TorrentsTrackers chat server"
)]
struct Cli {
    /// Path to the TOML config file. Defaults to `./tt-chat-server.toml`.
    #[arg(long, short, default_value = "tt-chat-server.toml")]
    config: PathBuf,
    /// Generate a starter config at the path and exit.
    #[arg(long)]
    init: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    if cli.init {
        config::write_starter(&cli.config)?;
        println!("starter config written to {}", cli.config.display());
        return Ok(());
    }

    let cfg = Config::load(&cli.config)
        .with_context(|| format!("loading config from {}", cli.config.display()))?;
    tracing::info!(
        "config: bind={}, server_name='{}', db={}",
        cfg.bind,
        cfg.server_name,
        cfg.db_path.display()
    );

    let db = db::Database::open(&cfg.db_path).context("open database")?;
    db.migrate().context("migrate database")?;

    let (broadcast_tx, _) = tokio::sync::broadcast::channel(256);
    let state = Arc::new(AppState {
        config: cfg.clone(),
        db,
        broadcast: broadcast_tx,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(handler::ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!("listening on {}", cfg.bind);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> &'static str {
    "tt-chat-server alive — connect via /ws"
}
