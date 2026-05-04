//! Database connection and high-level handle.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::Result;
use crate::migrations;

/// Thread-safe wrapper around a SQLite connection.
///
/// `rusqlite::Connection` is `!Sync`, so we wrap it in a `Mutex` to allow the
/// `Database` handle to be shared across tasks. This is fine for a desktop
/// app — contention is negligible. If we ever need multiple readers, we'll
/// switch to a connection pool.
pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) the database file at the given path and apply all
    /// pending migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        configure(&conn)?;
        migrations::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory database — used in tests.
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        configure(&conn)?;
        migrations::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_in_memory_and_migrates() {
        let db = Database::open_in_memory().expect("open");
        let conn = db.conn.lock().unwrap();
        let v: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key='schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v, migrations::CURRENT_VERSION.to_string());
    }
}
