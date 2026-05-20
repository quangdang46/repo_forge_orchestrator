# `rfo` — ADDITION.md

> Additions to PLAN.md to enable AI-driven multi-repo orchestration.
>
> Scope: 5 additions only. Everything else is deferred.
> Principle: same product discipline as PLAN.md — if a human or AI agent won't
> use it in the first month, it is deferred.

---

## A1. Multi-repo targeting

### Problem

`rfo sweep agent --plan` and `rfo train run` only operate on one repo at a time.
An AI agent handling many repos must call each separately with no coordination.

### New flags

```bash
# Target by pattern
rfo sweep agent --plan --repos "owner/*"
rfo train run --repos "owner/frontend-*,owner/backend-*"

# Target by filter
rfo train run --filter "tag:needs-fmt"
rfo train run --filter "health:<50"
rfo sweep agent --plan --filter "has:clippy-warnings"

# Target all managed repos (requires --dry-run on first use)
rfo train run --all --dry-run
rfo train run --all
```

### Rules

```text
--all requires --dry-run on first use or interactive confirmation.
--filter expressions: single condition only in v1, composable post-v1.
Multi-repo sweep creates one plan per repo, never a combined plan.
Concurrency bounded: default 4 repos at a time, configurable.
```

### Config addition

```toml
[agent]
max_concurrency = 4
require_dry_run_for_all = true
```

---

## A2. Inbox queue (minimal)

### Problem

`rfo inbox` shows a prioritized list but has no queue mechanic.
An AI agent has no safe way to consume repos one at a time without
re-reading the whole list each step.

### New commands

```bash
# Get the next inbox item (safe to poll)
rfo inbox next

# Mark an item as done after processing
rfo inbox done <item-id>
```

### Output of `rfo inbox next`

```json
{
  "id": "inbox-001",
  "repo": "owner/repo",
  "priority": 85,
  "reason": "3 open PRs need review, CI failing",
  "suggested_action": "rfo context owner/repo --pr 42"
}
```

Returns empty object `{}` when queue is empty — safe for polling.

### Rules

```text
No locking, no TTL, no claim/release in v1.
Single agent assumed. Multi-agent coordination is post-v1.
done persists to SQLite so the item does not reappear on next run.
```

---

## A3. MCP tools

### Problem

PLAN.md defines MCP resources (read-only). An AI agent also needs MCP tools —
actions it can invoke without dropping to CLI.

### New MCP tools

```text
rfo://tools/inbox-next       → next inbox item
rfo://tools/inbox-done       → mark item done
rfo://tools/plan-create      → create a plan for a repo
rfo://tools/plan-apply       → apply a plan (requires confirmed: true)
rfo://tools/plan-list        → list pending plans
rfo://tools/sweep-agent-plan → run sweep agent, return plan_id
rfo://tools/train-run        → run deterministic fixers, return plan_id
```

### Tool call contract

```json
{
  "tool": "plan-apply",
  "input": {
    "plan_id": "plan-abc123",
    "confirmed": true
  },
  "output": {
    "run_id": "run-xyz789",
    "status": "applied",
    "gates_passed": true
  }
}
```

### Rules

```text
No tool applies changes without confirmed: true.
Every mutating tool creates a run record in SQLite.
Errors return structured JSON, not plain text.
AI must pass confirmed: true explicitly — no default.
```

---

## A4. NDJSON progress for multi-repo runs

### Problem

When AI runs multi-repo operations it needs machine-readable progress —
not human-formatted terminal output.

### Event stream (--output json)

```jsonc
{"event":"batch_start","repos":12,"action":"train","dry_run":false,"ts":"..."}
{"event":"repo_start","repo":"owner/repo1","ts":"..."}
{"event":"plan_created","repo":"owner/repo1","plan_id":"plan-abc","risk":"LOW","ts":"..."}
{"event":"gates_passed","repo":"owner/repo1","plan_id":"plan-abc","ts":"..."}
{"event":"applied","repo":"owner/repo1","run_id":"run-xyz","ts":"..."}
{"event":"repo_done","repo":"owner/repo1","status":"ok","ts":"..."}
{"event":"gates_failed","repo":"owner/repo2","reason":"secret_detected","ts":"..."}
{"event":"repo_done","repo":"owner/repo2","status":"skipped","ts":"..."}
{"event":"batch_done","applied":8,"skipped":2,"failed":1,"ts":"..."}
```

### Rules

```text
Enabled with --output json on any multi-repo command.
Human output is unchanged when --output is omitted.
Extends the existing NDJSON output mode in PLAN.md — not a new system.
```

---

## A5. Plan auto-approve for low-risk batch runs

### Problem

In a multi-repo run, requiring interactive confirmation for every LOW risk plan
makes automation impractical. MEDIUM and HIGH should still require human review.

### New flag

```bash
rfo train run --repos "owner/*" --auto-approve low
rfo sweep agent --plan --repos "owner/*" --auto-approve low
```

### Approval levels

| Flag | LOW | MEDIUM | HIGH |
|---|---|---|---|
| (default, none) | prompt | prompt | prompt |
| `--auto-approve low` | auto | prompt | never |

HIGH risk is never auto-approved regardless of flag.

### Rules

```text
Auto-approve only triggers if all quality gates pass.
Auto-approve decisions are recorded in run history.
Can be set in config to avoid repeating on every command.
```

### Config addition

```toml
[agent.approval]
auto_approve = "none"    # none | low
```

---

## Deferred (not in v1)

```text
rfo agent-loop           — AI can orchestrate itself via MCP tools
inbox claim/release/TTL  — only needed for multi-agent, post-v1
failure patterns         — AI can infer patterns from rfo failures output
session rollback         — use rfo rollback <run-id> per run instead
--auto-approve medium    — MEDIUM still needs human review in v1
composable --filter      — single condition sufficient for v1
```

---

## Compatibility with PLAN.md

```text
No existing ru-compatible command is changed.
All additions are additive flags or new commands.
Safety gates, plan/apply, secret scan, denylist still required.
HIGH risk auto-approve never allowed.
SQLite remains the source of truth.
```

---

## Backlog

```text
add-1  --repos / --filter / --all flags on sweep and train
add-2  inbox next + inbox done
add-3  MCP tools schema and implementation
add-4  --output json NDJSON event stream for multi-repo
add-5  --auto-approve low flag and config
```

---

## Roadmap placement

| Addition | PLAN.md phase |
|---|---|
| A1 multi-repo targeting | Phase 2 (alongside sync engine) |
| A2 inbox queue minimal | Phase 4 (alongside inbox) |
| A3 MCP tools | Phase 5 (alongside MCP resources) |
| A4 NDJSON progress | Phase 2 (output mode extension) |
| A5 auto-approve low | Phase 3 (alongside plan/apply) |