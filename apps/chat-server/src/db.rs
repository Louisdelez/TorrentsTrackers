use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use tt_chat::ChatMessage;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id              TEXT PRIMARY KEY,
                channel         TEXT NOT NULL,
                author_pubkey   TEXT NOT NULL,
                content         TEXT NOT NULL,
                reply_to        TEXT,
                sent_at         TEXT NOT NULL,
                signature       TEXT NOT NULL,
                received_at     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_messages_channel_sent_at
                ON messages(channel, sent_at);

            CREATE TABLE IF NOT EXISTS bans (
                pubkey      TEXT PRIMARY KEY,
                reason      TEXT,
                banned_at   TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn insert_message(&self, m: &ChatMessage) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO messages
             (id, channel, author_pubkey, content, reply_to, sent_at, signature, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                m.id.to_string(),
                m.channel,
                m.author_pubkey,
                m.content,
                m.reply_to.map(|u| u.to_string()),
                m.sent_at.to_rfc3339(),
                m.signature,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn history(
        &self,
        channel: &str,
        before: Option<DateTime<Utc>>,
        limit: u32,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let (sql, args): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match before {
            Some(t) => (
                "SELECT id, channel, author_pubkey, content, reply_to, sent_at, signature
                 FROM messages WHERE channel = ?1 AND sent_at < ?2
                 ORDER BY sent_at DESC LIMIT ?3",
                vec![
                    Box::new(channel.to_string()),
                    Box::new(t.to_rfc3339()),
                    Box::new(limit as i64),
                ],
            ),
            None => (
                "SELECT id, channel, author_pubkey, content, reply_to, sent_at, signature
                 FROM messages WHERE channel = ?1
                 ORDER BY sent_at DESC LIMIT ?2",
                vec![Box::new(channel.to_string()), Box::new(limit as i64)],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let params_slice: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| &**b).collect();
        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            let id_s: String = row.get(0)?;
            let id = uuid::Uuid::parse_str(&id_s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let reply: Option<String> = row.get(4)?;
            let reply_to = reply
                .as_deref()
                .map(uuid::Uuid::parse_str)
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
            let sent_at_s: String = row.get(5)?;
            let sent_at = DateTime::parse_from_rfc3339(&sent_at_s)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);
            Ok(ChatMessage {
                id,
                channel: row.get(1)?,
                author_pubkey: row.get(2)?,
                content: row.get(3)?,
                reply_to,
                sent_at,
                signature: row.get(6)?,
            })
        })?;
        let mut out: Vec<ChatMessage> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        // Return chronological so the client can append directly.
        out.reverse();
        Ok(out)
    }

    pub fn is_banned(&self, pubkey_hex: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM bans WHERE pubkey = ?1)",
            params![pubkey_hex],
            |r| r.get(0),
        )?;
        Ok(exists != 0)
    }
}
