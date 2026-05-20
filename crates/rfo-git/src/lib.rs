//! Git operations for rfo.
//!
//! Read queries use `gix` (pure Rust, fast). Mutations use shell-out to
//! `git` for full ref/refspec/credential support. All mutations acquire
//! an `fs4` lock first.

pub mod lock;
pub mod mutation;
pub mod read;
pub mod status;

pub use status::RepoStatus;
