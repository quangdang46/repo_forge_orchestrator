//! OpenAI Codex provider.
//!
//! Invokes `codex exec` with configurable args. Parses the JSON output and
//! returns a structured [`InvokeOutput`]. Handles missing binaries, non-zero
//! exits, timeouts, and bad output.
//!
//! Note: Codex is deprecated / EOL'd as a standalone product as of early 2025.
//! We keep the provider here for parity with the v1 spec but mark it
//! [`ProviderAvailability::Deprecated`].

use std::path::PathBuf;
use std::pin::Pin;

use anyhow::{Result, anyhow};

use crate::traits::{InvokeOptions, InvokeOutput, Provider};

/// OpenAI Codex provider.
///
/// Spawns `codex exec`, pipes the prompt on stdin, then collects JSON events.
pub struct CodexProvider {
    program: PathBuf,
}

impl CodexProvider {
    /// Default provider using `codex` from PATH.
    pub fn new() -> Self {
        Self {
            program: PathBuf::from("codex"),
        }
    }

    /// Override the binary path.
    pub fn with_program(mut self, program: impl Into<PathBuf>) -> Self {
        self.program = program.into();
        self
    }
}

impl Default for CodexProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn is_available(&self) -> bool {
        // Codex is EOL'd; even if the binary exists we consider it unavailable.
        false
    }

    fn invoke<'a>(
        &'a self,
        prompt: &'a str,
        opts: &'a InvokeOptions,
    ) -> Pin<Box<dyn Future<Output = Result<InvokeOutput>> + Send + 'a>> {
        Box::pin(async move {
            // Always fail — Codex is deprecated.
            let _ = (prompt, opts);
            Err(anyhow!(
                "codex is deprecated (EOL'd as of 2025); use claude instead"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_codex() {
        assert_eq!(CodexProvider::new().name(), "codex");
    }

    #[test]
    fn is_always_unavailable() {
        // Even with a valid-looking program path, Codex is EOL'd.
        let p = CodexProvider::new().with_program("/usr/bin/codex");
        assert!(!p.is_available());
    }

    #[tokio::test]
    async fn errors_with_deprecated_message() {
        let p = CodexProvider::new();
        let opts = InvokeOptions::default();
        let err = p.invoke("hi", &opts).await.unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("deprecated"), "unexpected: {msg}");
    }
}
