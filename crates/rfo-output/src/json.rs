//! JSON output helpers.

/// Write a pretty-printed JSON value to stdout.
pub fn print_pretty(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

/// Write a compact JSON value to stdout.
pub fn print_compact(value: &serde_json::Value) {
    println!("{}", serde_json::to_string(value).unwrap_or_default());
}
