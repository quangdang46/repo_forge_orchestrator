//! Test utilities and fixtures for rfo.
//!
//! Provides:
//! - In-memory SQLite databases for testing
//! - Temporary repo fixtures
//! - Fake providers
//! - Assertion helpers

pub mod fakes;
pub mod fixtures;

use anyhow::Result;
use rusqlite::Connection;

/// Create an in-memory SQLite database with migrations applied.
pub fn test_db() -> Result<Connection> {
    rfo_state::open_memory()
}
