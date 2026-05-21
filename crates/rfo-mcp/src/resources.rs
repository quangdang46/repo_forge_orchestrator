//! MCP resource definitions.
//!
//! Exposes rfo state as MCP resources for AI agents.

use serde::{Deserialize, Serialize};

/// A resource definition for the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// List all available rfo resources.
pub fn list_resources() -> Vec<ResourceDef> {
    vec![
        ResourceDef {
            uri: "rfo://repos".into(),
            name: "repos".into(),
            description: "All tracked repos".into(),
            mime_type: "application/json".into(),
        },
        ResourceDef {
            uri: "rfo://inbox".into(),
            name: "inbox".into(),
            description: "Items needing attention".into(),
            mime_type: "application/json".into(),
        },
        ResourceDef {
            uri: "rfo://context/{owner}/{repo}".into(),
            name: "context".into(),
            description: "Context for a specific repo".into(),
            mime_type: "application/json".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_resources_returns_expected() {
        let resources = list_resources();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"rfo://repos"));
        assert!(uris.contains(&"rfo://inbox"));
    }
}
