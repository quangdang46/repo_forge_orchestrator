//! MCP server for rfo.
//!
//! stdio transport via rmcp.
//! Tools: rfo.status, rfo.inbox, rfo.health, rfo.context, rfo.review_plan,
//!        rfo.job_status, rfo.run_timeline.
//! Resources: rfo://repos, rfo://inbox, rfo://context/{owner}/{repo},
//!            rfo://runs/{id}/timeline.
//! Mutation tools create plans, never apply directly.

pub mod resources;
pub mod server;
pub mod tools;
