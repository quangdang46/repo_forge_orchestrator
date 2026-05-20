//! Configuration loading and validation for rfo.
//!
//! Reads `$XDG_CONFIG_HOME/rfo/config.toml`, applies defaults, validates.

pub mod loader;
pub mod schema;

pub use loader::load_config;
pub use schema::AppConfig;
