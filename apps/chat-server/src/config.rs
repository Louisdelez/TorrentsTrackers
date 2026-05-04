use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Address to bind to (e.g. `0.0.0.0:6970`).
    pub bind: String,
    /// Stable server identifier surfaced in the `auth_accepted` handshake.
    pub server_id: String,
    /// Display name shown to clients on connect.
    pub server_name: String,
    /// SQLite file path.
    pub db_path: PathBuf,
    /// History rows returned per `History` request.
    #[serde(default = "default_history_limit")]
    pub history_default_limit: u32,
    /// Per-connection rate limit on outgoing chat messages (msgs/min).
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_min: u32,
}

fn default_history_limit() -> u32 {
    200
}
fn default_rate_limit() -> u32 {
    60
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }
}

const STARTER: &str = r#"# tt-chat-server config

# Bind address. Use 0.0.0.0 to accept connections from outside the host.
bind = "127.0.0.1:6970"

# Stable identifier for this server (surfaced to clients on connect).
# Generate with `uuidgen` or pick anything unique to your community.
server_id = "ttchat-example-001"

# Display name shown to clients.
server_name = "Local TT chat"

# SQLite database path.
db_path = "tt-chat-server.sqlite"

# How many messages a `History` request returns by default.
history_default_limit = 200

# Per-connection outgoing message rate (msgs/min). 0 disables.
rate_limit_per_min = 60
"#;

pub fn write_starter(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("{} already exists; refusing to overwrite", path.display());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, STARTER)?;
    Ok(())
}
