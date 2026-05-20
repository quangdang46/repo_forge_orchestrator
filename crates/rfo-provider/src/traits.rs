//! Provider trait definition.

use anyhow::Result;
use serde_json::Value;

/// AI provider trait.
#[allow(async_fn_in_trait)]
pub trait Provider: Send + Sync {
    /// Provider name (e.g., "claude", "codex").
    fn name(&self) -> &str;

    /// Check if the provider binary is available.
    fn is_available(&self) -> bool;

    /// Invoke the provider with a prompt and return structured output.
    async fn invoke(&self, prompt: &str, args: &[String]) -> Result<Value>;
}
