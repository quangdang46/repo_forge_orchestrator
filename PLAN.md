# `rfo` — Repo Forge Orchestrator

> GitHub-first Rust rewrite and product evolution of `ru` (`repo_updater`)
>
> Repo: `github.com/quangdang46/repo_forge`  
> Reference source: `github.com/Dicklesworthstone/repo_updater`  
> Binary / crate: **`rfo`**  
> Author: `quangdang46`  
> Status: **PLAN.md — pragmatic v1 draft**  
> Core principle: **do not over-engineer; every feature must be used by either a human developer or an AI/MCP agent.**

---

## 0. Product thesis

`rfo` is a **GitHub-first repo orchestration tool**.

It starts from the useful idea of `ru`: keep many repos synced and inspectable.  
But the product goal is larger:

```text
rfo helps a developer or AI agent know what needs attention,
act safely, inspect what happened, and recover from failure.
```

The main user flow should be simple:

```text
rfo inbox
  -> see what needs attention

rfo context <repo>
  -> understand the repo / issue / PR

rfo review plan / rfo sweep agent --plan
  -> see proposed changes before applying

rfo apply <plan-id>
  -> perform safe automation

rfo run timeline <run-id>
  -> understand what happened

rfo rollback <run-id>
  -> recover when possible
```

The product should **not** become a pile of clever commands nobody uses.

---

## 1. Product discipline

A feature is accepted only if it directly improves one of these flows.

### 1.1 Human daily workflow

A human user needs answers to:

```text
What needs attention?
What should I run next?
Is this safe?
What happened?
How do I recover?
```

### 1.2 AI/MCP workflow

An AI/MCP agent needs answers to:

```text
What is the repo context?
What action is allowed?
What changed?
Did quality gates pass?
Can the change be reviewed or rolled back?
```

### 1.3 Deferral rule

If a feature does not clearly improve:

```text
inbox
context
plan/apply
rollback
timeline
failure recovery
safe deterministic maintenance
```

then it is deferred.

---


## 2. Feature + UX parity contract

`rfo` v1 is not only feature-compatible with `ru`; it must also preserve the **user experience** of `ru`.

This means:

```text
same core commands
same important flags
same defaults
same output modes
same exit-code meaning
same non-interactive script behavior
same interactive prompts where applicable
same config expectations where applicable
same operational flow for sync/status/review/sweep/fork/conflict
```

### 2.1 Default mode must feel like `ru`

When a user runs the equivalent command, the default experience should be familiar:

```bash
ru sync
rfo sync
```

The Rust implementation may be completely different internally, but the user-visible behavior should remain compatible.

### 2.2 Enhancements are additive, not replacements

New `rfo` capabilities are allowed only if they do not replace or break the old UX.

Examples:

```text
OK: rfo sync behaves like ru sync, but also records SQLite run history.
OK: rfo status behaves like ru status, but can additionally support --json improvements.
OK: rfo run timeline <id> is new and additive.
OK: rfo inbox is new and additive.

NOT OK: rfo sync changes default behavior in a way ru users/scripts do not expect.
NOT OK: replacing ru review flow with a totally different required flow.
NOT OK: removing a ru flag because the new architecture does not need it internally.
```

### 2.3 Compatibility tiers

Every `ru` behavior must be classified before v1 launch:

| Tier | Meaning |
|---|---|
| Exact parity | Same command/flag/default/output/exit behavior |
| Compatible parity | Same user outcome; minor formatting/internal differences allowed |
| Documented difference | Intentional difference with migration note |
| Deferred | Not allowed for core `ru` behavior unless explicitly accepted |

### 2.4 Parity audit artifact

Before v1, create and maintain:

```text
docs/RU_PARITY.md
```

It must include:

```text
ru command
ru flags
ru default behavior
ru output modes
ru exit codes
ru interactive UX
rfo equivalent
parity tier
test coverage
notes
```

### 2.5 Compatibility test rule

A feature is not considered ported until at least one of these is true:

```text
existing ru E2E test passes against rfo
new compatibility test is added
documented difference is approved
```

### 2.6 New UX rule

New product features such as inbox, context packs, timeline, failure memory, and Tiny PR Train are allowed, but they must be **additional entrypoints**.

They must not make the old `ru` style workflow harder.


## 3. Goals

### 2.1 Core goals

1. **Feature + UX parity-first Rust rewrite of `ru`**
   - Preserve `ru` user-visible features 1:1.
   - Preserve `ru` UX 1:1 where users already rely on it.
   - Preserve command names, important aliases, flags, defaults, prompts, output modes, error semantics, and exit-code behavior.
   - Preserve JSON / NDJSON / TOON output compatibility where useful.
   - Carry over black-box E2E tests.
   - New `rfo` features must be additive and must not break the existing `ru` flow.

2. **GitHub-only v1**
   - v1 supports GitHub.com.
   - No GitLab, Gitea, Forgejo, Bitbucket in v1.
   - No generic multi-forge abstraction in v1.

3. **SQLite source of truth**
   - SQLite is the durable state.
   - Legacy `state.json` is import-only.
   - NDJSON is export/log compatibility only.

4. **Safe local Git automation**
   - Use `gix` for read-only queries.
   - Use shell-out `git` for mutations.
   - Use repo-level file locks before mutations.
   - No shell string interpolation.

5. **Human-usable GitHub command center**
   - `rfo inbox`
   - `rfo health`
   - `rfo ci autopsy`
   - `rfo failures`

6. **AI/MCP-safe context and action model**
   - `rfo context`
   - MCP resources/tools for status, inbox, context, and plans.
   - AI should receive structured context, not blind filesystem access.

7. **Plan before risky mutation**
   - Risky operations create plans.
   - Plans are inspectable.
   - Applying a plan is explicit.
   - Rollback metadata is stored where possible.

8. **Small safe automation**
   - Deterministic tiny PR train.
   - Review/sweep through plan + gates only.
   - No giant autonomous refactor mode.

---


## 4. RU parity baseline — do not drop

This section is a guardrail.  
When simplifying the product, do **not** accidentally remove functionality that already exists or is expected from `ru`.

### 3.1 Required parity commands

| `ru` behavior | `rfo` v1 command | Status |
|---|---|---|
| initialize config/state | `rfo init` | required |
| add repo | `rfo add <repo>` | required |
| remove repo | `rfo remove <repo>` | required |
| list repos | `rfo list` | required |
| import repo list | `rfo import <file>` | required |
| sync repos | `rfo sync` | required |
| show repo status | `rfo status` | required |
| prune missing/removed repos | `rfo prune` | required |
| doctor/setup checks | `rfo doctor` | required |
| self update | `rfo self-update` | required |
| config inspect/edit | `rfo config` | required |
| fork status | `rfo fork status` | required if present in `ru` workflow |
| fork sync | `rfo fork sync` | required if present in `ru` workflow |
| fork clean | `rfo fork clean` | required if present in `ru` workflow |
| commit sweep | `rfo sweep commit` | required |
| agent sweep | `rfo sweep agent --plan` | required, safety-gated |
| review | `rfo review plan/apply` | required, safety-gated |
| robot docs | `rfo robot-docs` | required if users depend on it |
| conflict handling | `rfo conflict ...` | required explicit v1 feature |

### 4.2 Feature + UX parity rule

Before changing, removing, renaming, or deferring a feature/UX behavior, check:

```text
Is this a feature users already expect from ru?
Is this a UX flow users already know from ru?
Is this flag/default/output/exit-code used by scripts?
Is this needed for compatibility tests?
Is this needed for safe sync/review/sweep/fork/conflict flow?
```

If yes, keep it in v1 with compatible UX, or explicitly document the accepted difference in `docs/RU_PARITY.md`.

### 3.3 Accepted simplification rule

Simplification is allowed only when it removes unnecessary abstraction, not user-visible capability.

Good simplification:

```text
GitHub-only instead of multi-forge.
No TUI until CLI/MCP is stable.
No policy auto-apply in v1.
```

Bad simplification:

```text
Dropping conflict handling.
Dropping config command.
Dropping fork commands if ru users rely on them.
Dropping robot-docs if scripts depend on it.
```


## 5. Non-goals for v1

These are intentionally deferred.

```text
No multi-forge support.
No web UI.
No WASM plugin system.
No semantic vector search.
No ML-based PR classifier.
No distributed SSH execution.
No huge autonomous refactor agent.
No complex policy auto-repair engine.
No health trend/radar dashboard in v1.
No generic analytics SQL console in v1.
No TUI unless CLI/MCP workflows are already solid.
```

---

## 6. Locked decisions

| Area | Decision |
|---|---|
| Product direction | GitHub repo orchestration |
| Forge support | GitHub-only v1 |
| State | SQLite source of truth |
| Legacy state | Import-only |
| Output | text, JSON, NDJSON, TOON |
| Git queries | `gix` |
| Git mutations | shell-out `git` |
| Locking | `fs4` repo-path locks |
| Runtime | `tokio` |
| CLI | `clap` v4 |
| Logging | `tracing` |
| HTTP client | `reqwest` + `rustls` |
| GitHub REST | `octocrab` |
| GitHub GraphQL | `cynic` |
| SQLite | `sqlx` |
| Migrations | `sqlx migrate` |
| Time | `jiff` |
| MCP | `rmcp` |
| Daemon HTTP, if kept | `axum` |
| Config schema | `schemars` |
| Tests | `assert_cmd`, `insta`, `proptest`, `tempfile`, `wiremock` |
| Benchmarks | `criterion` |
| Release | `cargo-dist` |
| Audit | `cargo-deny`, `cargo-audit` |
| Rust edition | Rust 2024 |
| MSRV | Rust 1.85+ |

---

## 7. V1 feature set

V1 should be powerful, but not bloated.

### 5.1 Core repo operations

```bash
rfo init
rfo add <repo>
rfo remove <repo>
rfo list
rfo import <file>
rfo sync
rfo status
rfo prune
rfo doctor
rfo self-update
rfo config
rfo fork status
rfo fork sync
rfo fork clean
rfo robot-docs
```

These are the parity/core commands.

---

### 5.2 Human command center

```bash
rfo inbox
rfo health [repo]
rfo ci autopsy <repo> [--pr <number>]
rfo failures
rfo failures explain <id>
```

Purpose:

```text
help the human know what needs attention,
why something is failing,
and what to do next.
```

---

### 5.3 Safety and recovery

```bash
rfo plan <command...>
rfo apply <plan-id>
rfo rollback <run-id>
rfo run list
rfo run show <run-id>
rfo run timeline <run-id>
```

Purpose:

```text
make risky automation inspectable,
make runs explainable,
and make recovery possible.
```

---


### 5.4 Conflict Resolver

```bash
rfo conflict list
rfo conflict explain <repo>
rfo conflict plan <repo>
rfo conflict abort <repo>
rfo conflict mark-resolved <repo>
```

Purpose:

```text
make merge/rebase/cherry-pick conflicts understandable and recoverable.
```

V1 scope:

```text
detect conflicted repos
show conflicted files
show current Git operation state
explain safe options
allow safe abort
allow human/AI to create a resolution plan
apply only with confirmation
run quality gates after resolution
```

Rules:

```text
never silently auto-resolve conflicts
never choose local/remote automatically without explicit user intent
AI may propose a resolution plan, but apply requires confirmation
abort must be easy and safe
```


### 5.5 AI/MCP context

```bash
rfo context <repo>
rfo context <repo> --issue <number>
rfo context <repo> --pr <number>
```

MCP should expose:

```text
rfo://repos
rfo://inbox
rfo://context/{owner}/{repo}
rfo://runs/{id}/timeline
```

Purpose:

```text
give AI/MCP agents structured, minimal, correct context.
```

---

### 5.6 Safe deterministic maintenance

```bash
rfo train plan
rfo train run
```

Only deterministic fixers in v1:

```text
cargo fmt
cargo clippy --fix
prettier
ruff --fix
go fmt
npm install --package-lock-only
schema regeneration
lockfile refresh
```

Rules:

```text
one small change per PR
no AI mega-PR
quality gates required
secret scan required
denylist enforced
```

---

### 5.7 Review and sweep

```bash
rfo review plan
rfo review apply <plan-id>
rfo sweep commit
rfo sweep agent --plan
```

Rules:

```text
review/sweep must go through plan/apply
AI edits must use isolated branch/worktree
quality gates must run
secret scan must run
denylist must be enforced
```

---

## 8. Features intentionally simplified

### 6.1 No separate "Changeset Capsule" abstraction

The idea is useful, but v1 should not introduce another name.

Use:

```text
Plan = proposed change package
Run = executed history
Rollback = recovery path
```

Commands:

```bash
rfo plan show <plan-id>
rfo apply <plan-id>
rfo run timeline <run-id>
rfo rollback <run-id>
```

---

### 6.2 No complex risk budget engine

v1 only needs simple risk classification.

```text
LOW
MEDIUM
HIGH
```

Risk reasons:

```text
touches many files
modifies CI workflow
modifies dependency file
deletes files
no tests changed
quality gates unavailable
similar failure happened recently
```

No complex policy/rules DSL yet.

---

### 6.3 Basic health only

Keep:

```bash
rfo health <repo>
```

Defer:

```text
30-day health radar
trend dashboard
historical charts
```

---

### 6.4 Policy check only

Keep:

```bash
rfo policy check
```

Defer:

```bash
rfo policy apply --safe
```

Reason:

```text
auto-repairing GitHub settings, branch protection, labels, and files can get complex.
Check first. Apply later.
```

---

### 6.5 Daemon is optional, not core

Daemon is useful only if it directly supports:

```text
GitHub webhook -> enqueue job -> update inbox/status/context
```

No dashboard daemon.
No complex background automation in v1.

---

## 9. Workspace layout

```text
repo_forge/
├── Cargo.toml
├── rust-toolchain.toml
├── deny.toml
├── .cargo/
│   └── config.toml
├── crates/
│   ├── rfo/              # binary CLI
│   ├── rfo-core/         # types, errors, repo specs, paths, ids, redaction
│   ├── rfo-config/       # config, XDG paths, validation, schema
│   ├── rfo-output/       # text/json/toon/ndjson renderers
│   ├── rfo-state/        # SQLite schema, migrations, queries
│   ├── rfo-git/          # gix read queries + git command mutations
│   ├── rfo-github/       # GitHub REST/GraphQL client
│   ├── rfo-jobs/         # lightweight durable jobs and events
│   ├── rfo-sync/         # sync/status/prune/import/add/remove/list
│   ├── rfo-context/      # context packs for human/AI/MCP
│   ├── rfo-review/       # review plan/apply
│   ├── rfo-sweep/        # commit/agent sweep
│   ├── rfo-provider/     # Claude/Codex providers
│   ├── rfo-mcp/          # MCP server
│   └── rfo-testkit/      # fixtures/fakes
├── xtask/
│   └── src/main.rs
├── docs/
│   ├── ARCHITECTURE.md
│   ├── CLI.md
│   ├── MCP.md
│   ├── SAFETY.md
│   └── SCHEMAS/
├── scripts/
│   ├── e2e/
│   └── benchmarks/
└── .github/
    └── workflows/
        ├── ci.yml
        ├── release.yml
        └── audit.yml
```

### 7.1 Deferred crates

Do not create these unless needed later:

```text
rfo-daemon      # only if webhook/background jobs are actually needed
rfo-tui         # only after CLI/MCP UX is stable
rfo-policy      # only if policy check grows large
rfo-analytics   # deferred
rfo-plugin      # deferred
```

---

## 10. Runtime dependencies

| Dependency | Required? | Notes |
|---|---:|---|
| `git` | Yes | Required for mutations |
| GitHub token | Yes for GitHub API | `GITHUB_TOKEN`, config token, or `gh` fallback |
| `gh` | Optional | Auth fallback only |
| `claude` | Optional | Required only for Claude provider |
| `codex` | Optional | Required only for Codex provider |
| Everything else | Embedded | HTTP, SQLite, MCP, core logic |

No v1 dependency on:

```text
tmux
ntm
gemini
ollama
br
bv
```

---

## 11. GitHub repo spec parser

Accepted v1 formats:

```text
owner/repo
github.com/owner/repo
https://github.com/owner/repo
https://github.com/owner/repo.git
git@github.com:owner/repo.git
owner/repo#branch
owner/repo as alias
```

Rejected v1 formats:

```text
gitlab:owner/repo
gitea://host/owner/repo
forgejo://host/owner/repo
bitbucket:owner/repo
```

Parsed type:

```rust
pub struct RepoSpec {
    pub host: String,          // default: github.com
    pub owner: String,
    pub name: String,
    pub branch: Option<String>,
    pub alias: Option<String>,
    pub clone_url: String,
}
```

---

## 12. Configuration

### 10.1 XDG paths

```text
$XDG_CONFIG_HOME/rfo/config.toml
$XDG_CONFIG_HOME/rfo/repos.list
$XDG_CONFIG_HOME/rfo/policies.yaml
$XDG_STATE_HOME/rfo/state.db
$XDG_STATE_HOME/rfo/logs/<run_id>/
$XDG_CACHE_HOME/rfo/
```

Defaults:

```text
~/.config/rfo/config.toml
~/.local/state/rfo/state.db
~/.cache/rfo/
```

### 10.2 Example config

```toml
[core]
projects_dir = "~/projects"
layout = "flat"
parallel = 8
timeout_secs = 30

[github]
host = "github.com"
auth = "auto" # env | gh | config-token | auto

[git]
update_strategy = "ff-only"
autostash = false
terminal_prompt = false

[jobs]
enabled = true
max_attempts = 3
retry_backoff = "exponential"
default_timeout_secs = 1800

[mcp]
enabled = true
stdio = true
sse = false
sse_port = 7300

[review]
provider = "claude"
quality_gates = "auto"

[providers.claude]
bin = "claude"
default_args = ["-p", "--output-format", "stream-json"]

[providers.codex]
bin = "codex"
default_args = ["exec"]

[safety]
secret_scan = "block"
require_plan_for_ai_apply = true
max_auto_apply_risk = "low"
```

---

## 13. SQLite state model

SQLite is the single source of truth.

### 11.1 Pragmas

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

### 11.2 Tables

```sql
CREATE TABLE repos (
    id              TEXT PRIMARY KEY,
    host            TEXT NOT NULL DEFAULT 'github.com',
    owner           TEXT NOT NULL,
    name            TEXT NOT NULL,
    branch          TEXT,
    alias           TEXT,
    clone_url       TEXT NOT NULL,
    local_path      TEXT NOT NULL,
    visibility      TEXT NOT NULL,
    default_branch  TEXT,
    archived        INTEGER NOT NULL DEFAULT 0,
    disabled        INTEGER NOT NULL DEFAULT 0,
    added_at        INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    UNIQUE(host, owner, name)
);

CREATE TABLE runs (
    id              TEXT PRIMARY KEY,
    command         TEXT NOT NULL,
    started_at      INTEGER NOT NULL,
    ended_at        INTEGER,
    exit_code       INTEGER,
    args_json       TEXT NOT NULL,
    user            TEXT,
    host            TEXT
);

CREATE TABLE run_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id          TEXT NOT NULL REFERENCES runs(id),
    ts              INTEGER NOT NULL,
    level           TEXT NOT NULL,
    message         TEXT NOT NULL,
    data_json       TEXT
);

CREATE TABLE sync_results (
    run_id          TEXT NOT NULL REFERENCES runs(id),
    repo_id         TEXT NOT NULL REFERENCES repos(id),
    action          TEXT NOT NULL,
    status          TEXT NOT NULL,
    duration_ms     INTEGER NOT NULL,
    error           TEXT,
    pre_oid         TEXT,
    post_oid        TEXT,
    PRIMARY KEY (run_id, repo_id)
);

CREATE TABLE jobs (
    id              TEXT PRIMARY KEY,
    kind            TEXT NOT NULL,
    status          TEXT NOT NULL,
    repo_id         TEXT REFERENCES repos(id),
    payload_json    TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    started_at      INTEGER,
    ended_at        INTEGER,
    attempts        INTEGER NOT NULL DEFAULT 0,
    max_attempts    INTEGER NOT NULL DEFAULT 3,
    error           TEXT,
    created_by      TEXT NOT NULL DEFAULT 'cli'
);

CREATE TABLE job_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id          TEXT NOT NULL REFERENCES jobs(id),
    ts              INTEGER NOT NULL,
    level           TEXT NOT NULL,
    message         TEXT NOT NULL,
    data_json       TEXT
);

CREATE TABLE plans (
    id              TEXT PRIMARY KEY,
    kind            TEXT NOT NULL,
    repo_id         TEXT REFERENCES repos(id),
    status          TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    applied_at      INTEGER,
    risk_class      TEXT,
    risk_reasons_json TEXT,
    plan_json       TEXT NOT NULL,
    rollback_json   TEXT
);

CREATE TABLE failures (
    id              TEXT PRIMARY KEY,
    fingerprint     TEXT NOT NULL UNIQUE,
    class           TEXT NOT NULL,
    first_seen_at   INTEGER NOT NULL,
    last_seen_at    INTEGER NOT NULL,
    count           INTEGER NOT NULL,
    suggested_fix   TEXT
);

CREATE TABLE repo_health_snapshots (
    id              TEXT PRIMARY KEY,
    repo_id         TEXT NOT NULL REFERENCES repos(id),
    ts              INTEGER NOT NULL,
    score           INTEGER NOT NULL,
    class           TEXT NOT NULL,
    details_json    TEXT NOT NULL
);

CREATE TABLE context_cache (
    id              TEXT PRIMARY KEY,
    repo_id         TEXT NOT NULL REFERENCES repos(id),
    kind            TEXT NOT NULL,
    cache_key       TEXT NOT NULL,
    generated_at    INTEGER NOT NULL,
    expires_at      INTEGER,
    content_json    TEXT NOT NULL,
    UNIQUE(repo_id, kind, cache_key)
);

CREATE TABLE audit_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ts              INTEGER NOT NULL,
    actor           TEXT NOT NULL,
    action          TEXT NOT NULL,
    target          TEXT,
    details_json    TEXT
);
```

### 11.3 Indexes

```sql
CREATE INDEX idx_repos_owner_name ON repos(owner, name);
CREATE INDEX idx_runs_started_at ON runs(started_at);
CREATE INDEX idx_run_events_run_ts ON run_events(run_id, ts);
CREATE INDEX idx_jobs_status_created ON jobs(status, created_at);
CREATE INDEX idx_job_events_job_ts ON job_events(job_id, ts);
CREATE INDEX idx_failures_class ON failures(class);
CREATE INDEX idx_health_repo_ts ON repo_health_snapshots(repo_id, ts);
CREATE INDEX idx_audit_ts ON audit_log(ts);
```

---

## 14. Core crate specs

## 12.1 `rfo-core`

Pure foundation crate.

Exports:

```rust
pub struct RepoSpec { ... }
pub fn parse_repo_spec(s: &str) -> Result<RepoSpec, ParseRepoSpecError>;

pub enum Layout {
    Flat,
    OwnerRepo,
    Full,
}

pub enum ExitCode {
    Ok = 0,
    Partial = 1,
    Conflicts = 2,
    System = 3,
    BadArgs = 4,
    Interrupted = 5,
}

pub enum RepoStatus {
    Current,
    Behind { n: u32 },
    Ahead { n: u32 },
    Diverged { ahead: u32, behind: u32 },
    Conflict,
    Dirty,
    Missing,
    Unknown,
}
```

Modules:

```text
repo_spec
paths
ids
exit_codes
redact
denylist
risk
```

---

## 12.2 `rfo-config`

Responsibilities:

```text
load config
resolve XDG paths
validate config
write default config
migrate from ru
generate schema
```

---

## 12.3 `rfo-output`

Output modes:

```rust
pub enum OutputMode {
    Text,
    Json,
    Toon,
    Ndjson,
}
```

Rules:

```text
JSON is stable and schema-backed.
NDJSON is append-friendly.
TOON is retained for compatibility.
Text is concise and human-readable.
```

---

## 12.4 `rfo-state`

Responsibilities:

```text
open SQLite
run migrations
store repos/runs/jobs/plans/failures/audit
provide typed queries
```

Important rule:

```text
SQLite records state and history.
fs4 repo locks protect actual Git mutations.
```

---

## 12.5 `rfo-git`

Read-only layer uses `gix`.

```rust
pub async fn status(repo: &Path) -> Result<RepoGitStatus>;
pub async fn ahead_behind(repo: &Path) -> Result<AheadBehind>;
pub async fn head_oid(repo: &Path) -> Result<Option<String>>;
pub async fn is_dirty(repo: &Path) -> Result<bool>;
pub async fn current_branch(repo: &Path) -> Result<Option<String>>;
```

Mutation layer uses shell-out `git`.

```rust
pub async fn fetch(repo: &Path, opts: FetchOpts) -> Result<GitCommandResult>;
pub async fn pull(repo: &Path, opts: PullOpts) -> Result<PullOutcome>;
pub async fn clone(url: &str, dest: &Path, opts: CloneOpts) -> Result<CloneOutcome>;
pub async fn commit(repo: &Path, files: &[PathBuf], msg: &str) -> Result<String>;
pub async fn push(repo: &Path, opts: PushOpts) -> Result<()>;
pub async fn reset_hard(repo: &Path, oid: &str) -> Result<()>;
```

Command rules:

```text
Use Command::arg only.
Set GIT_TERMINAL_PROMPT=0.
Set LC_ALL=C where useful.
Use --no-pager.
Capture stdout/stderr.
Timeout external commands.
Classify errors.
```

---

## 12.6 `rfo-github`

Responsibilities:

```text
auth discovery
repo lookup
issue/PR listing
check status lookup
CI log fetching
rate limit handling
comments/reviews
small PR creation for train
```

No generic forge trait in v1.

---

## 12.7 `rfo-jobs`

Lightweight durable jobs.

Use jobs for:

```text
foreground CLI tracking
daemon/webhook later
MCP long-running actions
retry/cancel/watch
```

Do not overbuild a distributed queue.

Job kinds:

```rust
pub enum JobKind {
    Sync,
    Status,
    InboxRefresh,
    Health,
    CiAutopsy,
    ConflictList,
    ConflictExplain,
    ConflictPlan,
    ConflictAbort,
    ReviewPlan,
    ReviewApply,
    SweepAgent,
    TrainPlan,
    TrainRun,
    ContextRefresh,
}
```

---

## 12.8 `rfo-sync`

Responsibilities:

```text
add/list/remove/import/prune
parallel sync
status
run records
NDJSON export
```

Sync flow:

```text
read repos
create run
for each repo:
  acquire fs4 lock
  clone if missing
  fetch
  calculate ahead/behind
  pull according to strategy
  record pre/post oid
  release lock
finish run
emit output
```

---


## 12.9 `rfo-conflict` or `rfo-sync::conflict`

Do not create a separate crate unless the implementation grows large.  
The first implementation can live inside `rfo-sync` or `rfo-git`.

Commands:

```bash
rfo conflict list
rfo conflict explain <repo>
rfo conflict plan <repo>
rfo conflict abort <repo>
rfo conflict mark-resolved <repo>
```

Responsibilities:

```text
detect merge/rebase/cherry-pick conflict state
list conflicted files
read conflict markers
show ours/theirs/base status where available
explain safe options
create a resolution plan
abort merge/rebase/cherry-pick safely
verify resolution before marking resolved
run quality gates after resolution
```

Non-goals:

```text
no blind auto-resolve
no automatic "take ours/theirs" without explicit flag
no large semantic merge engine in v1
```

Resolution flow:

```text
rfo sync
  -> conflict detected

rfo conflict explain owner/repo
  -> show files and options

rfo conflict plan owner/repo
  -> create plan, optionally AI-assisted

rfo apply <plan-id>
  -> apply patch / user resolution

quality gates
  -> pass before final commit/push

rfo conflict mark-resolved owner/repo
  -> verify no conflict markers and Git state is clean
```


## 12.9 `rfo-context`

Context packs for humans and AI.

```bash
rfo context owner/repo
rfo context owner/repo --issue 123
rfo context owner/repo --pr 42
```

Context includes:

```text
local git status
GitHub issue/PR metadata
CI status
recent failures
quality gates
important files
risk notes
suggested commands
```

---

## 12.10 `rfo-review`

Commands:

```bash
rfo review plan
rfo review apply <plan-id>
```

Rules:

```text
always plan first
show diff/risk/gates
AI edits isolated
quality gates required
secret scan required
```

---

## 12.11 `rfo-sweep`

Commands:

```bash
rfo sweep commit
rfo sweep agent --plan
```

Rules:

```text
no silent mutation
denylist checked before staging
secret scan before commit
quality gates before push
```

---

## 12.12 `rfo-provider`

v1 providers:

```text
ClaudeCodeProvider
CodexProvider
```

No v1:

```text
GeminiProvider
OllamaProvider
```

Claude:

```bash
claude -p --output-format stream-json
```

Codex:

```bash
codex exec
```

---

## 12.13 `rfo-mcp`

MCP should expose only useful tools/resources.

Tools:

```text
rfo.status
rfo.inbox
rfo.health
rfo.context
rfo.review_plan
rfo.job_status
rfo.run_timeline
```

Resources:

```text
rfo://repos
rfo://inbox
rfo://context/{owner}/{repo}
rfo://runs/{id}/timeline
```

Mutation through MCP should prefer plan creation, not direct apply.

---

## 15. Human command center details

## 13.1 `rfo inbox`

Purpose:

```text
show what needs attention across all repos
```

Sources:

```text
GitHub PRs
GitHub issues
failed checks
local dirty repos
local diverged repos
failed previous runs
basic policy violations
```

Priority classes:

```text
CRITICAL
HIGH
NORMAL
LOW
INFO
```

Example:

```text
CRITICAL
  owner/api     PR #42 failing CI for 3 days
  owner/core    local branch diverged from origin/main

HIGH
  owner/web     issue #88 labeled bug + priority/high
```

Optional next action:

```text
Next: rfo ci autopsy owner/api --pr 42
Next: rfo review plan --issue 88
```

Keep next actions simple. Do not build a complex `rfo do A1` system in v1 unless clearly useful.

---

## 13.2 `rfo health`

Purpose:

```text
quickly explain repo state
```

Score:

```text
90-100 Excellent
75-89  Healthy
50-74  Needs attention
25-49  Risky
0-24   Critical
```

Penalty examples:

```text
failed CI
local diverged
merge conflict
dirty worktree
stale PRs
missing basic files
recent failed job
```

Output must explain the score.

---

## 13.3 `rfo ci autopsy`

Purpose:

```text
make failed GitHub Actions/checks understandable
```

MVP behavior:

```text
fetch failed jobs
fetch failed step logs
extract error snippets
detect ecosystem
identify likely file/test
suggest next command
record failure class
```

No AI required for MVP.

---

## 13.4 `rfo failures`

Purpose:

```text
remember repeated problems and show fixes
```

Failure classes:

```text
auth_error
rate_limited
merge_conflict
dirty_worktree
network_timeout
missing_git
missing_provider
quality_gate_failed
secret_scan_blocked
github_permission_denied
```

Example:

```text
Failure: GitHub permission denied

Seen 7 times across 3 repos.

Likely fix:
  gh auth refresh -s repo
  or set GITHUB_TOKEN with repo scope
```

---


## 16. Conflict Resolver details

Conflict handling is a first-class v1 feature, not just an error state.

### 14.1 `rfo conflict list`

Shows all repos currently in conflict or unfinished Git operation state.

Detect:

```text
MERGE_HEAD
REBASE_HEAD
CHERRY_PICK_HEAD
REVERT_HEAD
unmerged index entries
conflict markers in files
```

### 14.2 `rfo conflict explain <repo>`

Shows:

```text
current Git operation: merge/rebase/cherry-pick/revert
conflicted files
ours/theirs status
last sync/review/sweep run that caused it
safe options
suggested next command
```

### 14.3 `rfo conflict plan <repo>`

Creates a plan for resolution.

Plan may be:

```text
manual guidance only
take ours/theirs for specific files, if explicitly requested
AI-proposed patch
abort operation
```

### 14.4 `rfo conflict abort <repo>`

Safely aborts the current operation when Git allows it:

```text
git merge --abort
git rebase --abort
git cherry-pick --abort
git revert --abort
```

Records the action in the run timeline and audit log.

### 14.5 `rfo conflict mark-resolved <repo>`

Verifies:

```text
no unmerged index entries
no conflict markers in tracked files
quality gates pass or are explicitly skipped
```

Then records the conflict as resolved.


## 17. Safety model

### 14.1 Principles

```text
Never silently perform risky mutation.
Prefer plan/apply for complex changes.
Record pre-state before mutation.
Use repo locks.
Use branch/worktree for AI changes.
Run quality gates.
Run secret scan.
Enforce denylist.
Record audit events.
Provide rollback or manual recovery guidance.
```

---

### 14.2 Plan

A plan contains:

```json
{
  "id": "...",
  "kind": "review|sweep|train|sync",
  "repo": "owner/repo",
  "summary": "...",
  "operations": [],
  "risk": {
    "class": "low|medium|high",
    "reasons": []
  },
  "rollback": {},
  "created_at": "..."
}
```

---

### 14.3 Apply

```bash
rfo apply <plan-id>
```

Rules:

```text
LOW risk can apply normally.
MEDIUM risk requires confirmation.
HIGH risk requires explicit --yes or is blocked depending config.
```

---

### 14.4 Rollback

```bash
rfo rollback <run-id>
```

v1 rollback support:

```text
reset repo to pre-run OID when safe
delete created branch when safe
show manual recovery steps when automatic rollback is unsafe
```

No fake guarantee. If rollback cannot be safely automated, say so.

---

### 14.5 Secret scan

Modes:

```text
off
warn
block
```

Default:

```text
block
```

Scan:

```text
staged files
generated patches
AI edits before commit
```

---

### 14.6 Denylist

Default protected paths:

```text
.env
.env.*
*.pem
*.key
id_rsa
id_ed25519
**/.git/**
**/target/**
**/node_modules/**
```

Checks:

```text
during plan validation
before staging
before commit
```

---

## 18. Quality gates

Auto-detected gates:

| Ecosystem | Gates |
|---|---|
| Rust | `cargo fmt --check`, `cargo test`, `cargo clippy` |
| Node | `npm test`, `npm run lint`, `npm run typecheck` |
| Python | `pytest`, `ruff`, `mypy` |
| Go | `go test ./...`, `go fmt` |
| Shell | `shellcheck` |

Gate result:

```json
{
  "name": "cargo test",
  "status": "passed|failed|skipped",
  "duration_ms": 12345,
  "stdout_path": "...",
  "stderr_path": "..."
}
```

---

## 19. Tiny PR Train

Purpose:

```text
produce small, safe, deterministic maintenance PRs
```

Commands:

```bash
rfo train plan
rfo train run
```

Supported v1 fixers:

```text
cargo fmt
cargo clippy --fix
prettier
ruff --fix
go fmt
npm install --package-lock-only
schema regeneration
lockfile refresh
```

Flow:

```text
discover safe fixer
create branch
run fixer
inspect diff
run gates
secret scan
commit
push
open PR
record run
```

Reject if:

```text
too many files changed
denylisted path touched
secret scan fails
quality gates fail
diff includes suspicious deletion
patch is too large
```

---

## 20. Basic policy check

Command:

```bash
rfo policy check
```

Example policy:

```yaml
required_files:
  - README.md
  - LICENSE
  - SECURITY.md

github_labels:
  - bug
  - enhancement
  - good first issue
  - priority/high
```

V1 behavior:

```text
check only
report violations
suggest fixes
do not auto-apply GitHub settings
```

Auto apply is post-v1.

---

## 21. MCP design

MCP is useful only if it makes AI safer and better.

### 18.1 Tool rules

```text
Read tools can return direct data.
Mutation tools should create plans.
Direct apply through MCP should be optional and explicit.
```

### 18.2 Tools

```text
rfo.status
rfo.inbox
rfo.health
rfo.context
rfo.review_plan
rfo.run_timeline
```

### 18.3 Resources

```text
rfo://repos
rfo://inbox
rfo://context/{owner}/{repo}
rfo://runs/{id}/timeline
```

---

## 22. Error handling

Errors should answer:

```text
what failed
why it likely failed
what rfo tried
what to do next
where to inspect details
```

Example:

```text
GitHub permission denied for owner/private-repo.

Likely cause:
  token does not have repo access.

Try:
  gh auth status
  gh auth refresh -s repo
  or set GITHUB_TOKEN with repo scope.

Details:
  rfo run timeline 01HX...
```

---

## 23. Output formats

### 20.1 Text

Default for humans.

### 20.2 JSON

Machine-readable and schema-backed.

### 20.3 NDJSON

Streaming logs and compatibility.

### 20.4 TOON

Retained for compatibility and compact structured output.

Not required to be byte-for-byte identical with `ru`.

---

## 24. Exit codes

```rust
pub enum ExitCode {
    Ok = 0,
    Partial = 1,
    Conflicts = 2,
    System = 3,
    BadArgs = 4,
    Interrupted = 5,
}
```

Rules:

```text
Ok: everything succeeded
Partial: some repos/jobs failed
Conflicts: Git conflicts occurred
System: tool/config/auth failure
BadArgs: invalid CLI usage
Interrupted: signal/user cancellation
```

---

## 25. Security

### 22.1 Token handling

```text
read token from env/config/gh
never log token
redact token-like strings
avoid storing plaintext tokens
```

### 22.2 Command execution

```text
no shell interpolation
use Command::arg
explicit working directory
timeouts
captured stdout/stderr
```

### 22.3 Audit log

Record:

```text
actor
action
target
timestamp
parameters
result
```

---

## 26. Testing strategy

| Layer | Tooling | Purpose |
|---|---|---|
| Unit | `cargo test` | pure logic |
| Property | `proptest` | repo parser, state transitions |
| Snapshot | `insta` | output compatibility |
| CLI integration | `assert_cmd` | command behavior |
| Git fixtures | `tempfile` + `git` | real Git semantics |
| GitHub mock | `wiremock` | REST/GraphQL behavior |
| E2E parity | carried `ru` scripts | compatibility |
| Bench | `criterion` | sync/parser performance |
| Audit | `cargo-deny`, `cargo-audit` | CI gates |

Required fixtures:

```text
clean repo
repo behind origin
repo ahead of origin
diverged repo
dirty repo
merge conflict repo
missing repo
failed CI mock
rate limited GitHub mock
private repo auth failure mock
```

---

## 27. Release strategy

Artifacts via `cargo-dist`:

```text
linux-x86_64-musl
linux-aarch64-musl
darwin-x86_64
darwin-aarch64
windows-x86_64
```

Distribution:

```text
GitHub Releases
cargo install rfo
Homebrew tap
Docker image if useful
install.sh download + verify
```

Self-update:

```bash
rfo self-update
```

Must verify checksum/signature where practical.

---


## 27.5 RU parity audit process

Before implementation work spreads into new product features, perform a focused `ru` audit.

Inputs:

```text
ru source code
ru README/docs
ru shell scripts
existing E2E tests
known user workflows
```

Outputs:

```text
docs/RU_PARITY.md
scripts/e2e/parity/
snapshot fixtures for text/json/toon/ndjson
accepted-differences.md if needed
```

Audit checklist:

```text
commands
subcommands
flags
environment variables
config files
state files
repo list formats
interactive prompts
default output
JSON output
TOON output
NDJSON output
exit codes
conflict behavior
fork behavior
review behavior
sweep behavior
robot-docs behavior
error messages important for scripts
```

Implementation rule:

```text
No new product feature is allowed to replace a ru-compatible workflow.
New features can only wrap, explain, or extend the old workflow.
```


## 28. Roadmap

## Phase 0 — Foundation

```text
workspace bootstrap
Rust 2024 / MSRV 1.85+
CI
rfo-core
rfo-config
rfo-output
SQLite migrations
repo parser
```

Backlog:

```text
rfo-1  workspace bootstrap
rfo-2  CI fmt/clippy/test/deny/audit
rfo-3  repo spec parser
rfo-4  XDG config
rfo-5  output modes
rfo-6  initial SQLite schema
```

---

## Phase 1 — GitHub + Git + State

```text
rfo-git
rfo-github
GitHub auth
repo add/remove/list/import
repo locks
```

Backlog:

```text
rfo-7   gix read queries
rfo-8   shell-out git wrapper
rfo-9   fs4 repo lock
rfo-10  GitHub auth
rfo-11  GitHub repo lookup
rfo-12  init/add/remove/list/import
```

---

## Phase 2 — Sync parity

```text
sync
status
prune
parallel engine
run records
NDJSON export
E2E parity
```

Backlog:

```text
rfo-13 sync engine
rfo-14 status engine
rfo-15 prune
rfo-16 run records
rfo-17 NDJSON compatibility
rfo-18 parity E2E
```

---

## Phase 3 — Timeline, failures, safety

```text
run timeline
failure memory
plan/apply
rollback metadata
risk classification
secret scan
denylist
```

Backlog:

```text
rfo-19 run events
rfo-20 run timeline
rfo-21 failure classification
rfo-22 plan/apply
rfo-23 rollback support
rfo-24 risk classification
rfo-25 denylist + secret scan
rfo-26 conflict list/explain/abort
rfo-27 conflict plan/mark-resolved
```

---

## Phase 4 — Human command center

```text
inbox
health
CI autopsy
next action suggestions
```

Backlog:

```text
rfo-26 GitHub issue/PR query
rfo-27 check status query
rfo-28 inbox scoring
rfo-29 health scoring
rfo-30 CI log autopsy
```

---

## Phase 5 — Context + MCP

```text
context packs
MCP tools
MCP resources
```

Backlog:

```text
rfo-31 context pack
rfo-32 MCP stdio
rfo-33 MCP resources
rfo-34 MCP tools
```

---

## Phase 6 — Review/sweep/provider

```text
Claude provider
Codex provider
review plan/apply
sweep commit
sweep agent
quality gates
```

Backlog:

```text
rfo-35 Claude provider
rfo-36 Codex provider
rfo-37 quality gates
rfo-38 review plan/apply
rfo-39 sweep commit
rfo-40 sweep agent
```

---

## Phase 7 — Tiny PR train + policy check

```text
train plan
train run
policy check
```

Backlog:

```text
rfo-41 train plan
rfo-42 train run
rfo-43 policy parser
rfo-44 policy check
```

---

## Phase 8 — Release

```text
doctor
self-update
cargo-dist
Homebrew
signed artifacts
docs
```

Backlog:

```text
rfo-45 doctor
rfo-46 self-update
rfo-47 cargo-dist release
rfo-48 Homebrew tap
rfo-49 docs
rfo-50 v1 launch checklist
```

---

## 29. Post-v1 parking lot

Only after v1 is stable:

```text
GitHub Enterprise hardening
GitLab/Gitea/Forgejo support
Gemini/Ollama providers
TUI
Daemon/webhook mode
Policy apply --safe
Repo health trend/radar
Semantic code search
Web UI
WASM plugins
Distributed SSH execution
Beads/bv integration
Advanced analytics SQL
```

---

## 30. Main risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---:|---:|---|
| Scope creep | High | High | Product discipline and deferred list |
| Git mutation bugs | Medium | High | shell-out git, locks, pre/post OID, rollback |
| AI unsafe changes | Medium | High | plan/apply, gates, secret scan, denylist |
| GitHub rate limits | Medium | Medium | backoff, caching, lower concurrency |
| Output drift | High | Medium | snapshot + E2E tests |
| Provider CLI changes | Medium | Medium | provider isolation |
| Windows path issues | Medium | Medium | Windows CI after core parity |

---

## 31. Definition of done for v1

v1 is ready when:

```text
1. Core repo commands work reliably.
2. SQLite is the source of truth.
3. `docs/RU_PARITY.md` maps every important `ru` feature and UX flow.
4. `ru` parity tests pass or accepted differences are documented.
4. sync/status/prune are stable.
5. run timeline explains what happened.
6. failure memory handles common errors.
7. plan/apply/rollback exists for risky operations.
8. conflict resolver can list, explain, abort, plan, and verify conflict resolution.
9. inbox gives useful GitHub next actions.
10. context packs are useful for AI/MCP.
11. review/sweep are safety-gated.
12. tiny PR train handles deterministic fixes only.
13. release pipeline produces usable artifacts.
15. docs cover install, config, safety, MCP, conflict handling, and troubleshooting.
```

---

## 32. Final product summary

```text
rfo is a GitHub-first repo orchestration tool written in Rust.

It replaces fragile shell-based multi-repo workflows with a SQLite-backed,
safe, inspectable command center for syncing repos, understanding what needs
attention, giving AI agents structured context, planning risky changes,
running safe automation, and recovering from failure.
```

---

## 33. Final mantra

```text
GitHub-only v1.
SQLite source of truth.
git CLI for mutations.
gix for queries.
Feature + UX parity with ru first.
Do not over-engineer.
Every new feature must help a human or an AI agent.
Inbox tells what matters.
Context tells what is true.
Plan before risky mutation.
Conflict handling is explicit, safe, and recoverable.
Timeline explains what happened.
Rollback when possible.
Small safe PRs beat giant autonomous changes.
```
