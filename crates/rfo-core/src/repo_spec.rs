//! GitHub repo spec parser.

use serde::{Deserialize, Serialize};

/// Parsed GitHub repository specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoSpec {
    pub host: String,
    pub owner: String,
    pub name: String,
    pub branch: Option<String>,
    pub alias: Option<String>,
    pub clone_url: String,
}

impl RepoSpec {
    /// Canonical display: `owner/name`.
    pub fn canonical(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}
