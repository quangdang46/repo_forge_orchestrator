//! ntm (Named Tmux Manager) robot-mode integration.
//!
//! Provides a Rust wrapper around the ntm robot-mode CLI for orchestrating
//! AI agent sessions. See <https://github.com/nicktorn89/ntm>.
//!
//! ## Robot-mode commands
//!
//! | Command | Purpose | Exit codes |
//! |---------|---------|------------|
//! | `ntm spawn` | Create session with agents | 0=ok, 1=error, 2=unavailable |
//! | `ntm send` | Send prompt to agents | 0=delivered, 1=partial, 2=failed |
//! | `ntm wait` | Wait for condition (idle/error) | 0=met, 1=timeout, 2=error, 3=agent-error |
//! | `ntm interrupt` | Send Ctrl+C | 0=sent |
//! | `ntm kill` | Destroy session | 0=ok |
//!
//! ## Session naming
//!
//! Sessions follow the pattern: `rfo-ai-{repo_name}-{timestamp}`

use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

/// Exit codes from ntm robot commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtmExit {
    Ok,
    Error,
    Unavailable,
    Partial,
    Failed,
    Timeout,
    AgentError,
}

impl NtmExit {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => NtmExit::Ok,
            1 => NtmExit::Error,
            2 => NtmExit::Unavailable,
            3 => NtmExit::AgentError,
            _ => NtmExit::Error,
        }
    }
}

impl std::fmt::Display for NtmExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NtmExit::Ok => write!(f, "ok"),
            NtmExit::Error => write!(f, "error"),
            NtmExit::Unavailable => write!(f, "unavailable"),
            NtmExit::Partial => write!(f, "partial"),
            NtmExit::Failed => write!(f, "failed"),
            NtmExit::Timeout => write!(f, "timeout"),
            NtmExit::AgentError => write!(f, "agent_error"),
        }
    }
}

/// Check whether ntm is available and supports robot mode.
pub fn check_available() -> Result<bool> {
    let output = Command::new("ntm")
        .arg("--version")
        .output()
        .context("checking ntm availability")?;
    Ok(output.status.success())
}

/// Spawn a new ntm session. The session name must be unique.
/// Returns the session name on success.
pub fn spawn_session(session: &str, provider: &str) -> Result<NtmExit> {
    let output = Command::new("ntm")
        .arg("--robot-spawn")
        .arg(session)
        .arg("--provider")
        .arg(provider)
        .output()
        .context("spawning ntm session")?;
    Ok(NtmExit::from_code(output.status.code().unwrap_or(-1)))
}

/// Send a prompt to an ntm session. The prompt is auto-chunked at 4KB
/// for tmux limits.
pub fn send_prompt(session: &str, prompt: &str) -> Result<NtmExit> {
    let output = Command::new("ntm")
        .arg("--robot-send")
        .arg(session)
        .arg("--data")
        .arg(prompt)
        .output()
        .context("sending prompt to ntm session")?;
    Ok(NtmExit::from_code(output.status.code().unwrap_or(-1)))
}

/// Wait for a condition on an ntm session. `condition` is either "idle" or "error".
pub fn wait_completion(session: &str, condition: &str, transition: bool) -> Result<NtmExit> {
    let output = Command::new("ntm")
        .arg("--robot-wait")
        .arg(session)
        .arg("--condition")
        .arg(condition)
        .arg(if transition { "--transition" } else { "" })
        .output()
        .context("waiting for ntm session")?;
    Ok(NtmExit::from_code(output.status.code().unwrap_or(-1)))
}

/// Interrupt an ntm session (sends Ctrl+C).
pub fn interrupt_session(session: &str) -> Result<NtmExit> {
    let output = Command::new("ntm")
        .arg("--robot-interrupt")
        .arg(session)
        .output()
        .context("interrupting ntm session")?;
    Ok(NtmExit::from_code(output.status.code().unwrap_or(-1)))
}

/// Kill an ntm session. Idempotent.
pub fn kill_session(session: &str) -> Result<NtmExit> {
    let output = Command::new("ntm")
        .arg("kill")
        .arg("-f")
        .arg(session)
        .output()
        .context("killing ntm session")?;
    Ok(NtmExit::from_code(output.status.code().unwrap_or(-1)))
}

/// Poll session health, returning true if the session is idle.
pub fn is_session_idle(session: &str) -> Result<bool> {
    let output = Command::new("ntm")
        .arg("--robot-health")
        .arg(session)
        .output()
        .context("checking ntm session health")?;
    if !output.status.success() {
        return Ok(false);
    }
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("parsing ntm health JSON")?;
    let panes = json
        .get("panes")
        .and_then(|p| p.as_array())
        .and_then(|p| p.first());
    let activity = panes
        .and_then(|p| p.get("activity"))
        .and_then(|a| a.as_str());
    Ok(activity == Some("idle"))
}

/// Wait for a session to reach idle state with polling.
pub fn wait_for_idle(session: &str, timeout: Duration) -> Result<NtmExit> {
    let interval = Duration::from_secs(10);
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if is_session_idle(session)? {
            return Ok(NtmExit::Ok);
        }
        if std::time::Instant::now() >= deadline {
            return Ok(NtmExit::Timeout);
        }
        std::thread::sleep(interval);
    }
}

/// Full session lifecycle: spawn → send → wait → kill.
/// Returns the final NtmExit from the wait.
pub fn run_session(
    session: &str,
    provider: &str,
    prompt: &str,
    timeout: Duration,
) -> Result<NtmExit> {
    match spawn_session(session, provider) {
        Ok(NtmExit::Ok) => {}
        Ok(code) => return Ok(code),
        Err(e) => return Err(e),
    }

    match send_prompt(session, prompt) {
        Ok(NtmExit::Ok | NtmExit::Partial) => {}
        Ok(code) => {
            let _ = kill_session(session);
            return Ok(code);
        }
        Err(e) => {
            let _ = kill_session(session);
            return Err(e);
        }
    }

    let wait_result = wait_for_idle(session, timeout);
    let _ = kill_session(session);
    wait_result
}

/// Build a session name from a repo identifier.
pub fn session_name(owner: &str, repo: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("rfo-ai-{}-{}-{}", owner, repo, ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntm_exit_from_code() {
        assert_eq!(NtmExit::from_code(0), NtmExit::Ok);
        assert_eq!(NtmExit::from_code(1), NtmExit::Error);
        assert_eq!(NtmExit::from_code(2), NtmExit::Unavailable);
    }

    #[test]
    fn session_name_format() {
        let name = session_name("owner", "repo");
        assert!(name.starts_with("rfo-ai-owner-repo-"));
        assert!(name.contains("-"));
    }
}
