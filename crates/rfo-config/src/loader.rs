//! Configuration file loader.

use crate::schema::AppConfig;
use anyhow::{Context, Result};
use std::path::Path;

/// Load config from a TOML file, falling back to defaults.
pub fn load_config(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    let config: AppConfig =
        toml::from_str(&raw).with_context(|| format!("parsing config from {}", path.display()))?;
    Ok(config)
}
