//! SQLite state management for rfo.
//!
//! Provides schema migrations, connection management, and typed queries.
//! SQLite is the single source of truth for all rfo state.

pub mod migrate;
pub mod queries;

use anyhow::Result;
use std::path::Path;

/// Open (or create) the rfo state database with migrations applied.
pub fn open_db(path: &Path) -> Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate::run(&conn)?;
    Ok(conn)
}
