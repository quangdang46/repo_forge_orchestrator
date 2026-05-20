//! Human-readable text output helpers.

/// Write a section header.
pub fn header(text: &str) {
    println!("{}\n{}", text, "─".repeat(text.len()))
}

/// Write a key-value pair.
pub fn kv(key: &str, value: &str) {
    println!("  {key}: {value}");
}
