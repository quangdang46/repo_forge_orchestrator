//! Run event log management.
//!
//! Append-only events emitted by an in-flight run. The event stream
//! drives both the user-facing timeline and the audit log.
//!
//! Fields per PLAN.md §13: run_id, ts, level, message, data_json.

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Severity of a run event. Maps directly to log levels surfaced by the
/// CLI text/json output paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for EventLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventLevel::Debug => f.write_str("debug"),
            EventLevel::Info => f.write_str("info"),
            EventLevel::Warn => f.write_str("warn"),
            EventLevel::Error => f.write_str("error"),
        }
    }
}

impl FromStr for EventLevel {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "debug" => Ok(EventLevel::Debug),
            "info" => Ok(EventLevel::Info),
            "warn" | "warning" => Ok(EventLevel::Warn),
            "error" | "err" => Ok(EventLevel::Error),
            other => anyhow::bail!("unknown event level: {other}"),
        }
    }
}

/// A single row from the `run_events` table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunEvent {
    pub id: i64,
    pub run_id: String,
    pub ts: i64,
    pub level: EventLevel,
    pub message: String,
    /// Optional JSON-encoded payload.
    pub data_json: Option<String>,
}

/// Append a new event to a run. The caller-supplied `data` is
/// serialised to JSON if `Some`.
pub fn append_event(
    conn: &Connection,
    run_id: &str,
    level: EventLevel,
    message: &str,
    data: Option<&serde_json::Value>,
) -> Result<i64> {
    let ts = now_secs();
    let data_json = match data {
        Some(v) => Some(serde_json::to_string(v).context("serialising event data")?),
        None => None,
    };
    conn.execute(
        "INSERT INTO run_events (run_id, ts, level, message, data_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![run_id, ts, level.to_string(), message, data_json],
    )
    .context("inserting run event")?;
    Ok(conn.last_insert_rowid())
}

/// Fetch all events for a run, oldest first.
pub fn events_for_run(conn: &Connection, run_id: &str) -> Result<Vec<RunEvent>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, ts, level, message, data_json
             FROM run_events WHERE run_id = ?1 ORDER BY ts ASC, id ASC",
        )
        .context("preparing events_for_run query")?;
    let rows = stmt
        .query_map(params![run_id], row_to_event)
        .context("querying run events")?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunEvent> {
    let level: String = row.get(3)?;
    let level = level.parse::<EventLevel>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })?;
    Ok(RunEvent {
        id: row.get(0)?,
        run_id: row.get(1)?,
        ts: row.get(2)?,
        level,
        message: row.get(4)?,
        data_json: row.get(5)?,
    })
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
