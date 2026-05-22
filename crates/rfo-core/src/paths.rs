//! XDG-aware path helpers for rfo.

use std::path::PathBuf;

/// Return the config directory: `$XDG_CONFIG_HOME/rfo` or `~/.config/rfo`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("rfo"))
}

/// Return the state directory: `$XDG_STATE_HOME/rfo` or `~/.local/state/rfo`.
/// On Windows, falls back to `%LOCALAPPDATA%\rfo` since `dirs::state_dir()`
/// returns `None`.
pub fn state_dir() -> Option<PathBuf> {
    dirs::state_dir()
        .or_else(|| {
            #[cfg(windows)]
            {
                dirs::data_local_dir()
            }
            #[cfg(not(windows))]
            {
                None
            }
        })
        .map(|p| p.join("rfo"))
}

/// Return the cache directory: `$XDG_CACHE_HOME/rfo` or `~/.cache/rfo`.
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("rfo"))
}
