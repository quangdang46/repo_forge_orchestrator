//! AI-powered auto-commit for dirty repos.
//!
//! Stateless: processes each repo independently without persistent state.

use anyhow::{Context, Result};
use rfo_git::read::is_dirty;
use rfo_ntm::{self, NtmExit};
use rfo_sync::manage::TrackedRepo;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Repository with uncommitted changes.
pub struct DirtyRepo {
    pub repo: TrackedRepo,
    pub path: PathBuf,
}

/// Scan all repos, return those with uncommitted changes.
pub fn get_dirty_repos(repos: &[TrackedRepo]) -> Vec<DirtyRepo> {
    repos
        .iter()
        .filter_map(|repo| {
            let path = Path::new(&repo.local_path);
            match is_dirty(path) {
                Ok(true) => Some(DirtyRepo {
                    repo: repo.clone(),
                    path: path.to_path_buf(),
                }),
                Ok(false) | Err(_) => None,
            }
        })
        .collect()
}

fn phase1_prompt(repo: &DirtyRepo) -> String {
    let path = &repo.path;
    let owner = &repo.repo.owner;
    let name = &repo.repo.name;

    let changed = changed_files_summary(path);
    let agents_content = read_repo_file(path, "AGENTS.md");
    let readme_content = read_repo_file(path, "README.md");

    format!(
        r#"You are working on {owner}/{name}.

## Your task
Analyze ALL uncommitted changes in this repository and produce a detailed, descriptive commit message.

## Rules
- Commit message should follow conventional commits (feat:, fix:, chore:, docs:, refactor:, test:, style:, perf:, ci:, build:)
- If multiple types of changes exist, use multiple lines or a squash approach
- DO NOT commit anything — only analyze and report
- If there are no changes to commit, say "nothing to commit" explicitly

## Changed files
{changed}

## AGENTS.md (if exists)
{agents_content}

## README.md (if exists)
{readme_content}

## Your output
Respond ONLY with a JSON object:
{{"commit_message": "<your commit message>", "analysis": "<brief analysis of what changed and why>", "files_changed": ["<list of files>"]}}

If nothing to commit, respond with:
{{"commit_message": "nothing to commit", "analysis": "no uncommitted changes found", "files_changed": []}}
"#
    )
}

fn phase2_prompt(repo: &DirtyRepo, commit_message: &str) -> String {
    let owner = &repo.repo.owner;
    let name = &repo.repo.name;

    format!(
        r#"You are working on {owner}/{name}.

## Task
Commit the current uncommitted changes with this message:

---
{commit_message}
---

## Rules
- Run: git add . && git commit -m "..."
- Then run: git push origin HEAD (push to the current branch)
- If push fails because of non-fast-forward, say "push rejected" and stop
- If there is nothing to commit, say "nothing to commit" and stop

## Respond with JSON only:
{{"status": "committed" | "push_rejected" | "nothing to commit", "commit_oid": "<first 8 chars of commit OID if committed>", "push_output": "<brief push output or rejection reason>"}}
"#
    )
}

fn changed_files_summary(repo_path: &Path) -> String {
    let out = Command::new("git")
        .args(["diff", "--stat", "--porcelain"])
        .current_dir(repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => " (could not enumerate changed files)".to_string(),
    }
}

fn read_repo_file(repo_path: &Path, filename: &str) -> String {
    std::fs::read_to_string(repo_path.join(filename))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| format!("({filename} not found)"))
}

fn combined_prompt(repo: &DirtyRepo, commit_message: &str) -> String {
    let path = &repo.path;
    let owner = &repo.repo.owner;
    let name = &repo.repo.name;
    let changed = changed_files_summary(path);

    format!(
        r#"You are working on {owner}/{name}.

## Task
1. Analyze ALL uncommitted changes and verify the commit message is appropriate
2. Commit with: git add . && git commit -m "..."
3. Push with: git push origin HEAD

## Commit message to use
{commit_message}

## Changed files
{changed}

## Rules
- Verify the commit message matches the actual changes
- If there are no changes, respond: {{"status": "nothing to commit", "commit_oid": "", "push_output": ""}}
- If push fails (non-fast-forward), respond: {{"status": "push_rejected", ...}}
- Respond ONLY with: {{"status": "committed" | "push_rejected" | "nothing to commit", "commit_oid": "<8-char OID or empty>", "push_output": "<output or reason>"}}
"#
    )
}

pub fn sync_repo(
    repo: &DirtyRepo,
    provider: &str,
    no_push: bool,
    timeout: Duration,
) -> Result<(String, NtmExit)> {
    let session = rfo_ntm::session_name(&repo.repo.owner, &repo.repo.name);

    let phase1 = phase1_prompt(repo);
    let phase1_response = rfo_ntm::run_session(&session, provider, &phase1, timeout)?;

    if !matches!(phase1_response, NtmExit::Ok) {
        return Ok((
            format!("phase1 failed: {}", phase1_response),
            phase1_response,
        ));
    }

    let commit_message = "auto-commit from ai-sync";

    let phase2 = if no_push {
        let path = &repo.path;
        simple_commit(path, commit_message)?
    } else {
        combined_prompt(repo, commit_message)
    };

    let final_exit = if no_push {
        NtmExit::Ok
    } else {
        rfo_ntm::run_session(&session, provider, &phase2, timeout)?
    };

    Ok((commit_message.to_string(), final_exit))
}

fn simple_commit(repo_path: &Path, message: &str) -> Result<String> {
    let add = Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .output()
        .context("git add")?;
    if !add.status.success() {
        anyhow::bail!("git add failed: {}", String::from_utf8_lossy(&add.stderr));
    }

    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .output()
        .context("git commit")?;
    if !commit.status.success() {
        let stderr = String::from_utf8_lossy(&commit.stderr);
        if stderr.contains("nothing to commit") {
            return Ok("nothing to commit".to_string());
        }
        anyhow::bail!("git commit failed: {stderr}");
    }

    let oid = String::from_utf8_lossy(&commit.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(8)
        .collect::<String>();

    Ok(format!("committed {}", oid))
}
