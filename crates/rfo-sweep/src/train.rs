//! Tiny PR Train implementation.
//!
//! Discovers safe deterministic fixers and creates a plan per repo.
//! One small change per PR. Quality gates + secret scan required.

use anyhow::Result;
use std::path::Path;

/// A discovered fixer for a repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixer {
    pub name: String,
    pub command: String,
    pub ecosystem: String,
}

/// A train plan for a single repo.
#[derive(Debug, Clone)]
pub struct TrainPlan {
    pub repo_id: String,
    pub fixers: Vec<Fixer>,
    pub estimated_risk: String,
}

/// Discover safe deterministic fixers for a repo.
pub fn discover_fixers(repo_path: &Path) -> Vec<Fixer> {
    let mut fixers = Vec::new();

    if repo_path.join("Cargo.toml").exists() {
        fixers.push(Fixer {
            name: "cargo fmt".into(),
            command: "cargo fmt".into(),
            ecosystem: "rust".into(),
        });
        fixers.push(Fixer {
            name: "cargo clippy --fix".into(),
            command: "cargo clippy --fix --allow-dirty --allow-staged".into(),
            ecosystem: "rust".into(),
        });
    }

    if repo_path.join("package.json").exists() {
        fixers.push(Fixer {
            name: "npm install --package-lock-only".into(),
            command: "npm install --package-lock-only".into(),
            ecosystem: "node".into(),
        });
    }

    if repo_path.join("go.mod").exists() {
        fixers.push(Fixer {
            name: "go fmt".into(),
            command: "go fmt ./...".into(),
            ecosystem: "go".into(),
        });
    }

    if repo_path.join("pyproject.toml").exists() || repo_path.join("setup.py").exists() {
        fixers.push(Fixer {
            name: "ruff check --fix".into(),
            command: "ruff check --fix .".into(),
            ecosystem: "python".into(),
        });
    }

    fixers
}

/// Create a train plan for a repo.
pub fn plan_train(repo_path: &Path, repo_id: &str) -> TrainPlan {
    let fixers = discover_fixers(repo_path);
    let risk = if fixers.is_empty() { "unknown" } else { "low" };
    TrainPlan {
        repo_id: repo_id.to_string(),
        fixers,
        estimated_risk: risk.into(),
    }
}

/// Run a train plan: execute each fixer in sequence.
/// Returns (succeeded, failed) counts.
pub fn run_train(repo_path: &Path, plan: &TrainPlan) -> Result<TrainResult> {
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for fixer in &plan.fixers {
        match run_fixer(repo_path, fixer) {
            Ok(()) => succeeded.push(fixer.name.clone()),
            Err(e) => failed.push((fixer.name.clone(), e.to_string())),
        }
    }

    Ok(TrainResult { succeeded, failed })
}

/// Result of running a train plan.
#[derive(Debug, Clone)]
pub struct TrainResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<(String, String)>,
}

fn run_fixer(repo_path: &Path, fixer: &Fixer) -> Result<()> {
    use std::process::Command;
    let parts: Vec<&str> = fixer.command.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("empty command");
    }
    let mut cmd = Command::new(parts[0]);
    cmd.current_dir(repo_path);
    for arg in &parts[1..] {
        cmd.arg(arg);
    }
    let out = cmd.output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("{} failed: {}", fixer.name, stderr);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn discovers_rust_fixers() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            b"[package]\nname = \"test\"\n",
        )
        .unwrap();
        let fixers = discover_fixers(tmp.path());
        assert!(!fixers.is_empty());
        assert!(fixers.iter().any(|f| f.name == "cargo fmt"));
    }

    #[test]
    fn discovers_node_fixers() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("package.json"), b"{}\n").unwrap();
        let fixers = discover_fixers(tmp.path());
        assert!(fixers.iter().any(|f| f.ecosystem == "node"));
    }

    #[test]
    fn plan_train_risk_is_low_when_fixers_found() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), b"").unwrap();
        let plan = plan_train(tmp.path(), "test-repo");
        assert_eq!(plan.estimated_risk, "low");
    }
}
