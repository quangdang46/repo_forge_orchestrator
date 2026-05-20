//! Durable job execution for rfo.
//!
//! Jobs, runs, run events, failure classification.
//! Every mutating command creates a run record.

pub mod events;
pub mod failure;
pub mod run;
