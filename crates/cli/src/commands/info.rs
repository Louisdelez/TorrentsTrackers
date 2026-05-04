use std::path::Path;

use anyhow::Result;
use tt_storage::Database;

pub fn run(db_path: &Path, db: &Database) -> Result<()> {
    let entries = db.count_entries()?;
    let sources = db.list_sources()?.len();
    let pools = db.list_pools()?.len();
    println!("TorrentsTrackers — local instance");
    println!("  data dir : {}", tt_storage::paths::data_dir()?.display());
    println!("  database : {}", db_path.display());
    println!("  sources  : {sources}");
    println!("  pools    : {pools}");
    println!("  entries  : {entries}");
    Ok(())
}
