//! AI provider abstraction for rfo.
//!
//! Providers are isolated behind a common trait.
//! Claude: claude -p --output-format stream-json
//! Codex: codex exec

pub mod claude;
pub mod codex;
pub mod traits;
