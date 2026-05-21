//! Test fixtures, fakes, and utilities for rfo.
//!
//! Reusable test helpers across the workspace. Every test that touches
//! the filesystem or state should use `tempfile::TempDir` via helpers here.

pub mod fixtures;
pub mod fakes;

pub use fixtures::{git_repo, rust_project};
