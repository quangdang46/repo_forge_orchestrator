//! MCP tool definitions.
//!
//! Exposes rfo operations as MCP tools for AI agents.
//! See ADDITION.md §A3 for the canonical tool list.

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
            name: "rfo_inbox_next".into(),
            description: "Get the next inbox item (safe to poll)".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "rfo_inbox_done".into(),
            description: "Mark an inbox item as done".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string", "description": "Inbox item id" }
                },
                "required": ["item_id"]
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
        ToolDef {
            name: "rfo_plan_create".into(),
            description: "Create a review plan for a repo".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" },
                    "summary": { "type": "string" },
                    "risk": { "type": "string", "enum": ["low", "medium", "high"] }
                },
                "required": ["repo"]
            }),
        },
        ToolDef {
            name: "rfo_plan_list".into(),
            description: "List existing review plans".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "Optional status filter" }
                }
            }),
        },
        ToolDef {
            name: "rfo_plan_apply".into(),
            description: "Apply a previously-approved review plan".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "plan_id": { "type": "string" }
                },
                "required": ["plan_id"]
            }),
        },
        ToolDef {
            name: "rfo_sweep_agent_plan".into(),
            description: "Plan a sweep on a repo without applying".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" }
                }
            }),
        },
        ToolDef {
            name: "rfo_train_run".into(),
            description: "Run a Tiny PR Train on a repo".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" },
                    "dry_run": { "type": "boolean", "default": false }
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
        assert!(names.contains(&"rfo_inbox_next"));
        assert!(names.contains(&"rfo_inbox_done"));
        assert!(names.contains(&"rfo_health"));
        assert!(names.contains(&"rfo_plan_create"));
        assert!(names.contains(&"rfo_plan_apply"));
        assert!(names.contains(&"rfo_sweep_agent_plan"));
        assert!(names.contains(&"rfo_train_run"));
    }

    #[test]
    fn all_tools_have_parameters() {
        let tools = list_tools();
        for tool in &tools {
            assert!(
                tool.parameters.is_object(),
                "{} has non-object params",
                tool.name
            );
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
        }
    }
}
