//! Durable job execution for rfo.
//!
//! Jobs, runs, run events, failure classification.
//! Every mutating command creates a run record.

pub mod events;
pub mod failure;
pub mod run;

// Re-exports for ergonomic public API
pub use events::{EventLevel, RunEvent, append_event, events_for_run};
pub use failure::{FailureClass, classify};
pub use run::{RunRecord, finalize_run, get_run, open_run, open_runs, recent_runs};
