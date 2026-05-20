//! Configuration loading and validation for rfo.
//!
//! Reads `$XDG_CONFIG_HOME/rfo/config.toml`, applies defaults, validates.

pub mod loader;
pub mod paths;
pub mod schema;
pub mod validate;

pub use loader::{load_config, load_default, write_default};
pub use paths::{ConfigPaths, default_config_toml, expand_tilde};
pub use schema::AppConfig;
pub use validate::validate;
