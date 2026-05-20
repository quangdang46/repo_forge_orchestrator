//! GitHub API client for rfo.
//!
//! Auth discovery, repo lookup, issue/PR/check queries.
//! Rate-limit handling with backoff.

pub mod auth;
pub mod checks;
pub mod client;
pub mod issues;
