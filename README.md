# rfo — Repo Forge Orchestrator

> **GitHub-first repo orchestration CLI for humans and AI agents.**
> Keep many repos synced, know what needs attention, and apply small safe automations
> with a plan-then-apply workflow.

[![CI](https://github.com/quangdang46/repo_forge_orchestrator/actions/workflows/ci.yml/badge.svg)](https://github.com/quangdang46/repo_forge_orchestrator/actions/workflows/ci.yml)
[![Release](https://github.com/quangdang46/repo_forge_orchestrator/actions/workflows/release.yml/badge.svg)](https://github.com/quangdang46/repo_forge_orchestrator/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](rust-toolchain.toml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## What is `rfo`?

`rfo` is a Rust CLI that orchestrates many GitHub repositories from one place:

- **Track** repos in a local SQLite database (the source of truth).
- **Sync, status, prune** — keep working copies up to date.
- **Inbox & health** — see what needs your attention, ranked by signal.
- **Plan → apply → rollback** for risky operations (review, sweep, train).
- **Safety gates** by default: secret scan, denylist, quality checks.
- **JSON output** on every read command for scripting and AI consumption.
- **MCP-friendly** so an AI agent gets structured context, not blind shell access.

It is the GitHub-first Rust evolution of the Python tool `ru` (`repo_updater`),
preserving the daily UX while adding inspectable runs and AI-safe primitives.

> **Status:** early development — public API and command flags may change before
> v1.0.

---

## Install

### Linux / macOS (x86_64, aarch64)

```bash
curl -fsSL https://raw.githubusercontent.com/quangdang46/repo_forge_orchestrator/main/install.sh | bash
```

The installer detects your platform, downloads the matching release archive
from GitHub, **verifies its SHA256**, and places `rfo` in `~/.local/bin`.

Optional environment overrides:

| Variable          | Default              | Purpose                              |
|-------------------|----------------------|--------------------------------------|
| `RFO_VERSION`     | `latest`             | Pin a specific tag, e.g. `v0.1.0`.   |
| `RFO_INSTALL_DIR` | `$HOME/.local/bin`   | Where to place the binary.           |
| `RFO_NO_VERIFY`   | unset                | Set to `1` to skip checksum (avoid). |
| `RFO_FORCE`       | unset                | Set to `1` to overwrite silently.    |

```bash
# Pin a version
RFO_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/quangdang46/repo_forge_orchestrator/main/install.sh | bash

# Install system-wide
RFO_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/quangdang46/repo_forge_orchestrator/main/install.sh | sudo -E bash
```

### Windows (x86_64, PowerShell 5.1+)

```powershell
irm https://raw.githubusercontent.com/quangdang46/repo_forge_orchestrator/main/install.ps1 | iex
```

Installs `rfo.exe` into `%LOCALAPPDATA%\Programs\rfo` and adds it to your user
`PATH`. Open a new terminal afterward.

Optional `$env:` overrides: `RFO_VERSION`, `RFO_INSTALL_DIR`,
`RFO_NO_VERIFY`, `RFO_NO_MODIFY_PATH`.

### Build from source

Requires [Rust 1.85+](rust-toolchain.toml).

```bash
git clone https://github.com/quangdang46/repo_forge_orchestrator.git
cd repo_forge
cargo build --release
./target/release/rfo --version
```

### Manual download

Each tagged release on GitHub publishes:

- `rfo-<target>.tar.xz` (Linux + macOS) and `rfo-<target>.zip` (Windows)
- `*.sha256` checksum sidecar for every artifact
- Universal `rfo-installer.sh` and `rfo-installer.ps1` (cargo-dist installers, version-pinned)

See <https://github.com/quangdang46/repo_forge_orchestrator/releases>.

---

## Quickstart

```bash
# 1. Initialize config + SQLite state directory
rfo init

# 2. Add some repos
rfo add quangdang46/repo_forge_orchestrator
rfo add https://github.com/torvalds/linux

# 3. Bulk import a list (one "owner/repo" per line)
rfo import repos.list

# 4. Sync everything
rfo sync

# 5. See the world
rfo status
rfo health
rfo inbox

# 6. Health-check the install
rfo doctor
```

---

## Commands

```text
rfo [--config-dir <DIR>] [--state-dir <DIR>] <COMMAND>
```

| Group         | Command                              | What it does                                     |
|---------------|--------------------------------------|--------------------------------------------------|
| Setup         | `rfo init`                           | Create config + SQLite state directory.          |
|               | `rfo doctor [--fix]`                 | Diagnose (and optionally repair) the install.    |
| Repo manage   | `rfo add <spec>`                     | Track a repo (`owner/repo` or full URL).         |
|               | `rfo remove <key>`                   | Stop tracking.                                   |
|               | `rfo list [--owner <o>]`             | List tracked repos.                              |
|               | `rfo import <file>`                  | Bulk add from a `repos.list` file.               |
|               | `rfo prune --archived --missing`     | Drop archived or missing repos.                  |
| Sync / status | `rfo sync [--strategy ff-only\|rebase\|merge]` | Update working copies.                  |
|               | `rfo status [<repo>]`                | Branch / ahead / behind / dirty per repo.        |
|               | `rfo health [<repo>]`                | Health score with `text` or `json` output.       |
|               | `rfo inbox`                          | Prioritized list of repos needing attention.     |
| Runs          | `rfo run list [--limit <N>]`         | Recent run records.                              |
|               | `rfo run show <run-id>`              | Inspect a single run.                            |
|               | `rfo run timeline <run-id>`          | Replay events for a run.                         |
| Conflicts     | `rfo conflict list`                  | Show repos in a merge/rebase/cherry-pick state.  |
|               | `rfo conflict explain <repo>`        | Explain the conflict.                            |
|               | `rfo conflict abort <repo>`          | Abort the operation.                             |
|               | `rfo conflict mark-resolved <repo>`  | Mark resolved.                                   |
| Review        | `rfo review plan <repo> [--summary] [--risk]` | Create a review plan.                   |
|               | `rfo review apply <plan-id>`         | Apply a plan after gates pass.                   |
|               | `rfo review list-plans`              | List pending plans.                              |
| Sweep         | `rfo sweep commit --message <msg>`   | Stage + commit with safety gates.                |
|               | `rfo sweep agent [--repo-id <id>]`   | Run the sweep agent on a repo.                   |

Run `rfo <command> --help` for full flags. Every command supporting structured
output accepts `--format json`.

---

## Common workflows

### Daily check-in

```bash
rfo sync                # update working copies
rfo inbox               # what needs me?
rfo status              # any repos dirty / behind?
```

### Reviewing a risky change

```bash
rfo review plan owner/repo --summary "Bump deps" --risk medium
rfo review list-plans
rfo review apply plan-abc123
rfo run timeline run-xyz789   # what actually happened
```

### AI sync across dirty repos

```bash
# Find repos with uncommitted changes
rfo status

# Auto-commit and push each dirty repo via AI
rfo ai-sync --provider claude
```

### Safe commit gating

```bash
rfo sweep commit --path . --message "chore: format"
# Refuses to commit if secret scan or denylist trigger.
```

### AI-friendly mode

Every read command supports `--format json` and writes one JSON value per line,
making `rfo` safe to drive from an LLM, a script, or an MCP host:

```bash
rfo status --format json | jq .
rfo inbox  --format json
rfo health --format json
```

---

## Configuration

`rfo` follows the XDG Base Directory spec.

| Path                         | Default                              | Override flag       |
|------------------------------|--------------------------------------|---------------------|
| Config directory             | `$XDG_CONFIG_HOME/rfo`               | `--config-dir <DIR>`|
| State directory (SQLite, runs) | `$XDG_STATE_HOME/rfo`              | `--state-dir <DIR>` |
| Cache directory              | `<state-dir>/cache`                  | (derived)           |
| Tracked working copies       | `<state-dir>/projects/<owner>/<repo>`| (derived)           |

On Windows, `dirs` resolves these to the standard per-user locations (e.g.
`%APPDATA%`, `%LOCALAPPDATA%`).

GitHub authentication is read from the standard environment variables
(`GITHUB_TOKEN` / `GH_TOKEN`) and from `gh auth status` when present.

---

## Output formats

| Format | Where        | Use                                    |
|--------|--------------|----------------------------------------|
| `text` | default      | Human-readable, color-aware terminal.  |
| `json` | `--format json` | One JSON document per line (NDJSON-friendly). |

Multi-repo runs additionally emit an NDJSON event stream with `--output json`
(see [`ADDITION.md`](ADDITION.md) §A4).

---

## Crates

`rfo` is a Cargo workspace. The user-facing binary lives in `crates/rfo`; the
rest are libraries kept small and focused.

```text
crates/
├── rfo/             # CLI entry point (the `rfo` binary)
├── rfo-core/        # shared types, errors, IDs
├── rfo-config/      # config file loader + XDG paths
├── rfo-state/       # SQLite schema, queries, health
├── rfo-output/      # text + JSON renderers
├── rfo-jobs/        # run records, events, timeline
├── rfo-git/         # local git: gix reads + shell-out mutations
├── rfo-github/      # GitHub API client (octocrab)
├── rfo-sync/        # add/remove/import/sync/status/prune
├── rfo-context/     # context packs for humans + AI
├── rfo-review/      # plan / apply / rollback workflow
├── rfo-sweep/       # sweep commit, sweep agent
├── rfo-ntm/         # ntm robot-mode integration
├── rfo-ai-sync/     # AI-powered auto-commit for dirty repos
├── rfo-dep-update/  # AI-powered dependency updates
├── rfo-provider/    # provider abstraction (GitHub-only in v1)
├── rfo-mcp/         # MCP server: resources + tools
└── rfo-testkit/     # shared test fixtures
xtask/               # project-level tasks (cargo xtask <task>)
```

---

## Building & contributing

```bash
# Format + lint + test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace

# Project-level helpers
cargo xtask --help
```

Continuous integration runs the same commands on Linux, macOS, and Windows for
each push and PR — see [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

Releases are tag-driven (`v*.*.*`) and produced by
[`cargo-dist`](https://github.com/axodotdev/cargo-dist) — see
[`.github/workflows/release.yml`](.github/workflows/release.yml).

---

## License

[MIT](LICENSE) © Trần Quang Đãng (`quangdang46`).
