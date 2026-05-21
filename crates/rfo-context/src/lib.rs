//! Context and cache for rfo.
//!
//! Provides structured repo context for AI/MCP agents.
//! Full context pack assembly is rfo-33.

pub mod cache;
pub mod pack;

pub use cache::ContextCache;
pub use pack::ContextPack;
