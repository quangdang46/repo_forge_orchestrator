//! Failure classification.
//!
//! When a run exits non-zero, we classify the root cause so the
//! orchestrator can decide whether to retry, escalate, or auto-rollback.
//!
//! Classes per PLAN.md §14: auth_error, rate_limited, merge_conflict,
//! dirty_worktree, network_timeout, missing_git, missing_provider,
//! quality_gate_failed, secret_scan_blocked, github_permission_denied.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of a run failure. Each variant maps to a specific
/// recovery strategy in the orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    AuthError,
    RateLimited,
    MergeConflict,
    DirtyWorktree,
    NetworkTimeout,
    MissingGit,
    MissingProvider,
    QualityGateFailed,
    SecretScanBlocked,
    GithubPermissionDenied,
}

impl fmt::Display for FailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{self:?}"));
        f.write_str(&s)
    }
}

/// Whether a failure class is retryable (transient) or not (permanent).
impl FailureClass {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FailureClass::RateLimited | FailureClass::NetworkTimeout | FailureClass::MergeConflict
        )
    }

    pub fn is_fatal(&self) -> bool {
        !self.is_retryable()
    }
}

/// Classify a non-zero exit code + stderr heuristics into a failure class.
///
/// This mirrors the git heuristic in `rfo_git::mutation::GitErrorKind` but
/// operates at the job level so it can also capture quality-gate and
/// secret-scan failures.
pub fn classify(exit_code: i32, stderr: &str) -> FailureClass {
    let s = stderr.to_ascii_lowercase();
    if s.contains("authentication") || s.contains("permission denied") || s.contains("403") {
        if s.contains("github") || s.contains("api.github.com") {
            FailureClass::GithubPermissionDenied
        } else {
            FailureClass::AuthError
        }
    } else if s.contains("rate limit") || s.contains("429") {
        FailureClass::RateLimited
    } else if s.contains("merge conflict") || s.contains("conflict in") {
        FailureClass::MergeConflict
    } else if s.contains("dirty")
        || s.contains("uncommitted changes")
        || s.contains("your local changes")
    {
        FailureClass::DirtyWorktree
    } else if s.contains("timed out") || s.contains("connection refused") || s.contains("network") {
        FailureClass::NetworkTimeout
    } else if s.contains("git: not found") || s.contains("'git' is not recognized") {
        FailureClass::MissingGit
    } else if s.contains("quality gate") || s.contains("clippy") || s.contains("test failed") {
        FailureClass::QualityGateFailed
    } else if s.contains("secret") || s.contains("blocked by secret scan") {
        FailureClass::SecretScanBlocked
    } else if s.contains("provider") && (s.contains("not found") || s.contains("missing")) {
        FailureClass::MissingProvider
    } else {
        // Fallback: use exit code heuristics
        match exit_code {
            1 => FailureClass::AuthError,
            2 => FailureClass::NetworkTimeout,
            _ => FailureClass::AuthError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_auth_error() {
        assert_eq!(
            classify(1, "Authentication failed for repository"),
            FailureClass::AuthError
        );
    }

    #[test]
    fn classify_github_permission_denied() {
        assert_eq!(
            classify(1, "github: Permission denied to api.github.com"),
            FailureClass::GithubPermissionDenied
        );
    }

    #[test]
    fn classify_rate_limited() {
        assert_eq!(
            classify(1, "rate limit exceeded, retry after 60s"),
            FailureClass::RateLimited
        );
    }

    #[test]
    fn classify_merge_conflict() {
        assert_eq!(
            classify(1, "CONFLICT (content): Merge conflict in src/main.rs"),
            FailureClass::MergeConflict
        );
    }

    #[test]
    fn classify_dirty_worktree() {
        assert_eq!(
            classify(1, "your local changes would be overwritten"),
            FailureClass::DirtyWorktree
        );
    }

    #[test]
    fn classify_network_timeout() {
        assert_eq!(
            classify(1, "fatal: connection timed out"),
            FailureClass::NetworkTimeout
        );
    }

    #[test]
    fn classify_missing_git() {
        assert_eq!(classify(127, "git: not found"), FailureClass::MissingGit);
    }

    #[test]
    fn classify_quality_gate() {
        assert_eq!(
            classify(1, "quality gate failed: clippy reported errors"),
            FailureClass::QualityGateFailed
        );
    }

    #[test]
    fn classify_secret_scan() {
        assert_eq!(
            classify(1, "blocked by secret scan: AWS key found"),
            FailureClass::SecretScanBlocked
        );
    }

    #[test]
    fn classify_missing_provider() {
        assert_eq!(
            classify(1, "provider not found: claude"),
            FailureClass::MissingProvider
        );
    }

    #[test]
    fn retryable_classes() {
        assert!(FailureClass::RateLimited.is_retryable());
        assert!(FailureClass::NetworkTimeout.is_retryable());
        assert!(FailureClass::MergeConflict.is_retryable());
        assert!(!FailureClass::AuthError.is_retryable());
        assert!(!FailureClass::SecretScanBlocked.is_retryable());
    }

    #[test]
    fn display_roundtrip() {
        for variant in [
            FailureClass::AuthError,
            FailureClass::RateLimited,
            FailureClass::MergeConflict,
            FailureClass::SecretScanBlocked,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FailureClass = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, back);
        }
    }
}
