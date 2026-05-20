//! Shared formatting utilities.

/// Emit a single NDJSON line.
pub fn emit_ndjson(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// Format a timestamp for display.
pub fn format_timestamp(ts: &time::OffsetDateTime) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}
