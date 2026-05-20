//! Database migrations for rfo.

use anyhow::Result;
use rusqlite::Connection;

/// Run all pending migrations.
pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _meta (key TEXT PRIMARY KEY, value TEXT);")?;
    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(CAST(value AS INTEGER), 0) FROM _meta WHERE key='version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    apply(conn, current)
}

fn apply(conn: &Connection, from: i64) -> Result<()> {
    let migrations: &[&str] = &[
        // v1: repos table
        "CREATE TABLE IF NOT EXISTS repos (
            id          TEXT PRIMARY KEY,
            owner       TEXT NOT NULL,
            name        TEXT NOT NULL,
            host        TEXT NOT NULL DEFAULT 'github.com',
            branch      TEXT,
            alias       TEXT,
            clone_url   TEXT NOT NULL,
            local_path  TEXT,
            added_at    TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
            removed_at  TEXT
        );",
    ];
    for (i, sql) in migrations.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > from {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT OR REPLACE INTO _meta (key, value) VALUES ('version', ?1)",
                [version.to_string()],
            )?;
        }
    }
    Ok(())
}
