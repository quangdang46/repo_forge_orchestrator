//! Output rendering for rfo.
//!
//! Provides consistent output in text, JSON, NDJSON, and TOON formats.
//! Each command produces structured data; this crate handles display.

pub mod format;
pub mod json;
pub mod text;

/// Output format selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Ndjson,
    Toon,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Ndjson => write!(f, "ndjson"),
            OutputFormat::Toon => write!(f, "toon"),
        }
    }
}

/// Parse an output format from a string (CLI flag).
pub fn parse_output_format(s: &str) -> Option<OutputFormat> {
    match s.to_ascii_lowercase().as_str() {
        "text" => Some(OutputFormat::Text),
        "json" => Some(OutputFormat::Json),
        "ndjson" => Some(OutputFormat::Ndjson),
        "toon" => Some(OutputFormat::Toon),
        _ => None,
    }
}
