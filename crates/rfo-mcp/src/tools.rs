//! MCP tool definitions.
//!
//! Exposes rfo operations as MCP tools for AI agents.

use serde::{Deserialize, Serialize};

/// A tool definition for the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// List all available rfo tools.
pub fn list_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "rfo_repos".into(),
            description: "List all tracked repos".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "rfo_inbox".into(),
            description: "Get the inbox with items needing attention".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "rfo_health".into(),
            description: "Get health status for a repo".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo_id": { "type": "string" }
                }
            }),
        },
        ToolDef {
            name: "rfo_context".into(),
            description: "Get context for a repo".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo_id": { "type": "string" }
                }
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_tools_returns_expected_tools() {
        let tools = list_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"rfo_repos"));
        assert!(names.contains(&"rfo_inbox"));
        assert!(names.contains(&"rfo_health"));
    }
}
