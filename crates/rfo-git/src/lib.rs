//! Git operations for rfo.
//!
//! Read queries use gix; mutations use shell-out to git via std::process.
//! All mutations acquire an fs4 lock first.

pub mod lock;
pub mod mutation;
pub mod read;
pub mod status;
