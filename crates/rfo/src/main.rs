//! rfo — GitHub-first repo orchestration CLI.
//!
//! Binary entry point. Logic lives in library crates; this file
//! wires clap commands to those functions.
//!
//! Exit codes:
//! - 0: success / healthy
//! - 1: one or more failures
//! - 64: usage error

mod doctor;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rfo_config::paths::ConfigPaths;
use rfo_sync::sync::SyncStrategy;

/// Output format for commands that support it.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "rfo", about = "GitHub-first repo orchestration CLI", version, long_about = None)]
struct Cli {
    /// Override the config directory (default: $XDG_CONFIG_HOME/rfo)
    #[arg(long, global = true)]
    config_dir: Option<PathBuf>,

    /// Override the state directory (default: $XDG_STATE_HOME/rfo)
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    // ── Repo management ──────────────────────────────────────────────
    /// Initialize config + SQLite state directory
    Init,

    /// Add a repo to tracking
    Add {
        /// Repo spec: owner/repo, github.com/owner/repo, https://..., etc.
        spec: String,
    },

    /// Remove a repo from tracking
    Remove {
        /// Repo key: owner/repo, alias, or id
        key: String,
    },

    /// List tracked repos
    List {
        /// Filter by owner
        #[arg(long)]
        owner: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Bulk import repos from a file (one "owner/repo" per line)
    Import {
        /// Path to repos.list file
        file: PathBuf,
    },

    // ── Sync / Status ────────────────────────────────────────────────
    /// Sync all tracked repos
    Sync {
        /// Sync strategy: ff-only (default), rebase, merge
        #[arg(long, value_enum, default_value_t = SyncStrategy::FfOnly)]
        strategy: SyncStrategy,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show status of tracked repos
    Status {
        /// Specific repo key
        repo: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Prune removed/missing repos
    Prune {
        /// Also prune archived repos
        #[arg(long)]
        archived: bool,
        /// Also prune missing repos
        #[arg(long)]
        missing: bool,
    },

    // ── Health / Inbox ───────────────────────────────────────────────
    /// Show health score for repos
    Health {
        /// Specific repo key
        repo: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show inbox (items needing attention)
    Inbox {
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    // ── Runs / Timeline ──────────────────────────────────────────────
    /// Run management commands
    Run {
        #[command(subcommand)]
        sub: RunCommands,
    },

    // ── Conflict ─────────────────────────────────────────────────────
    /// Conflict resolver commands
    Conflict {
        #[command(subcommand)]
        sub: ConflictCommands,
    },

    // ── Review ───────────────────────────────────────────────────────
    /// Review plan/apply commands
    Review {
        #[command(subcommand)]
        sub: ReviewCommands,
    },

    // ── Sweep ────────────────────────────────────────────────────────
    /// Sweep commands (commit, agent)
    Sweep {
        #[command(subcommand)]
        sub: SweepCommands,
    },

    // ── Train ────────────────────────────────────────────────────────
    /// Tiny PR Train commands
    Train {
        #[command(subcommand)]
        sub: TrainCommands,
    },

    // ── Doctor ───────────────────────────────────────────────────────
    /// Diagnose installation health
    Doctor {
        /// Apply repairs
        #[arg(long)]
        fix: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
enum RunCommands {
    /// List recent runs
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show a specific run
    Show { run_id: String },
    /// Show timeline for a run
    Timeline { run_id: String },
}

#[derive(Debug, Subcommand)]
enum ConflictCommands {
    /// List conflicted repos
    List,
    /// Explain a conflict
    Explain { repo: String },
    /// Abort a conflict
    Abort { repo: String },
    /// Mark a conflict as resolved
    MarkResolved { repo: String },
}

#[derive(Debug, Subcommand)]
enum ReviewCommands {
    /// Create a review plan
    Plan {
        /// Repo key
        repo: String,
        /// Plan description
        #[arg(long)]
        summary: Option<String>,
        /// Risk level: low, medium, high
        #[arg(long)]
        risk: Option<String>,
    },
    /// Approve a pending plan so it can be applied
    Approve { plan_id: String },
    /// Reject a pending or approved plan
    Reject { plan_id: String },
    /// Apply a previously-approved review plan
    Apply { plan_id: String },
    /// Roll back a previously-applied plan
    Rollback { plan_id: String },
    /// List plans
    ListPlans,
}

#[derive(Debug, Subcommand)]
enum SweepCommands {
    /// Sweep commit with safety checks
    Commit {
        /// Repo path
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Commit message
        #[arg(long)]
        message: String,
    },
    /// Run sweep agent on a repo
    Agent {
        /// Repo path
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Repo id for tracking
        #[arg(long)]
        repo_id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum TrainCommands {
    /// Plan a train run
    Plan {
        /// Repo path
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Repo id
        #[arg(long)]
        repo_id: Option<String>,
    },
    /// Execute a train run
    Run {
        /// Repo path
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Repo id
        #[arg(long)]
        repo_id: Option<String>,
    },
}

/// Parse a `--risk` flag value into a [`rfo_review::plan::PlanRisk`].
///
/// Accepts case-insensitive `low` / `medium` / `high`. Rejecting unknown
/// strings here keeps the (otherwise opaque) DB column from accumulating
/// values that the risk classifier can't understand.
fn parse_risk(s: &str) -> Result<rfo_review::plan::PlanRisk> {
    match s.to_ascii_lowercase().as_str() {
        "low" => Ok(rfo_review::plan::PlanRisk::Low),
        "medium" | "med" => Ok(rfo_review::plan::PlanRisk::Medium),
        "high" => Ok(rfo_review::plan::PlanRisk::High),
        other => anyhow::bail!("unknown risk class {other:?} (expected low|medium|high)"),
    }
}

/// Build a `repo.id -> "owner/name"` map for friendlier text rendering.
///
/// Used by commands like `sync` and `health` whose underlying results carry
/// only the repo UUID. Returns an empty map on any DB error — callers fall
/// back to the raw id, so missing labels never break the command.
fn repo_labels_by_id(conn: &rfo_state::Connection) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let Ok(mut stmt) = conn.prepare("SELECT id, owner, name FROM repos") else {
        return map;
    };
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    });
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            map.insert(r.0, format!("{}/{}", r.1, r.2));
        }
    }
    map
}

fn resolve_paths(cli: &Cli) -> Result<ConfigPaths> {
    match (&cli.config_dir, &cli.state_dir) {
        (Some(config_dir), Some(state_dir)) => {
            let cache_dir = state_dir.join("cache");
            Ok(ConfigPaths {
                config_dir: config_dir.clone(),
                state_dir: state_dir.clone(),
                cache_dir,
            })
        }
        _ => ConfigPaths::discover(),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use rfo_sync::manage;
    use rfo_sync::prune;
    use rfo_sync::status;
    use rfo_sync::sync;

    let cli = Cli::parse();
    let paths = resolve_paths(&cli)?;
    let db_path = paths.state_db();

    match cli.command {
        // ── Repo management ──
        Commands::Init => {
            let created = manage::init(&paths).context("initializing rfo")?;
            if created {
                eprintln!("Initialized: {}", paths.config_dir.display());
            } else {
                eprintln!("Already initialized: {}", paths.config_dir.display());
                eprintln!("  Config:  {}", paths.config_toml().display());
                eprintln!("  State:   {}", db_path.display());
            }
        }

        Commands::Add { spec } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let projects_dir = paths.state_dir.join("projects");
            let repo = manage::add(&conn, &spec, &projects_dir).context("adding repo")?;
            eprintln!("Added: {}/{} (id={})", repo.owner, repo.name, repo.id);
        }

        Commands::Remove { key } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let repo = manage::remove(&conn, &key).context("removing repo")?;
            eprintln!("Removed: {}/{}", repo.owner, repo.name);
        }

        Commands::List { owner, format } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let repos = manage::list(&conn, owner.as_deref()).context("listing repos")?;
            if repos.is_empty() {
                eprintln!("No tracked repos. Use 'rfo add <spec>' to add one.");
            } else {
                for repo in repos {
                    match format {
                        OutputFormat::Text => println!("{}", repo),
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string(&repo)?);
                        }
                    }
                }
            }
        }

        Commands::Import { file } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let projects_dir = paths.state_dir.join("projects");
            let (added, skipped, errors) =
                manage::import(&conn, &file, &projects_dir).context("importing repos")?;
            if !added.is_empty() {
                eprintln!("Added {} repos", added.len());
            }
            if !skipped.is_empty() {
                eprintln!("Skipped {} duplicates", skipped.len());
            }
            if !errors.is_empty() {
                for (line, err) in &errors {
                    eprintln!("Error: {line}: {err}");
                }
            }
        }

        // ── Sync / Status ──
        Commands::Sync { strategy, format } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let opts = sync::SyncOptions {
                strategy,
                autostash: false,
                timeout_secs: 30,
            };
            // Snapshot repos before sync so we can render `owner/name` in the
            // text output even if a row is later mutated/removed.
            let repo_labels = repo_labels_by_id(&conn);
            let results = sync::sync_all(&conn, &opts).context("syncing repos")?;
            for r in &results {
                match format {
                    OutputFormat::Text => {
                        let label = repo_labels
                            .get(&r.repo_id)
                            .cloned()
                            .unwrap_or_else(|| r.repo_id.clone());
                        println!("{} action={} status={}", label, r.action, r.status);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(r)?);
                    }
                }
            }
        }

        Commands::Status { repo, format } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let statuses: Vec<status::RepoStatus> = match repo {
                Some(key) => {
                    let r = status::status_repo(&conn, &key)?;
                    vec![r]
                }
                None => status::status_all(&conn)?,
            };
            for s in &statuses {
                match format {
                    OutputFormat::Text => {
                        let dirty = if s.is_dirty { " (dirty)" } else { "" };
                        println!(
                            "{}/{}: {}{} ahead={} behind={}",
                            s.owner,
                            s.name,
                            s.branch.as_deref().unwrap_or("HEAD"),
                            dirty,
                            s.ahead,
                            s.behind
                        );
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(s)?);
                    }
                }
            }
        }

        Commands::Prune { archived, missing } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let mut pruned: Vec<prune::PruneResult> = Vec::new();
            if archived {
                let results = prune::prune_archived(&conn)?;
                pruned.extend(results);
            }
            if missing {
                let results = prune::prune_missing(&conn)?;
                pruned.extend(results);
            }
            if !pruned.is_empty() {
                eprintln!("Pruned {} repos:", pruned.len());
                for p in &pruned {
                    eprintln!("  {}/{}", p.owner, p.name);
                }
            } else {
                eprintln!("Nothing to prune.");
            }
        }

        // ── Health / Inbox ──
        Commands::Health { repo, format } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let snapshots = match repo {
                Some(key) => {
                    let found = manage::find_repo(&conn, &key)?;
                    vec![rfo_state::queries::score_repo_health(&conn, &found.id)?]
                }
                None => rfo_state::queries::score_all_health(&conn)?,
            };
            let repo_labels = repo_labels_by_id(&conn);
            for snap in &snapshots {
                match format {
                    OutputFormat::Text => {
                        let label = repo_labels
                            .get(&snap.repo_id)
                            .cloned()
                            .unwrap_or_else(|| snap.repo_id.clone());
                        println!("{}: score={} class={}", label, snap.score, snap.class);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(snap)?);
                    }
                }
            }
        }

        Commands::Inbox { format } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let items = rfo_state::queries::compute_inbox(&conn)?;
            if items.is_empty() {
                eprintln!("Inbox is empty.");
            }
            for item in &items {
                match format {
                    OutputFormat::Text => {
                        println!(
                            "{}/{}: priority={} {}",
                            item.owner, item.name, item.priority, item.reason
                        );
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(item)?);
                    }
                }
            }
        }

        // ── Runs / Timeline ──
        Commands::Run { sub } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            match sub {
                RunCommands::List { limit } => {
                    let runs = rfo_jobs::recent_runs(&conn, limit)?;
                    for run in &runs {
                        let status = if run.is_finished() {
                            format!("exit={}", run.exit_code.unwrap_or(0))
                        } else {
                            "running".into()
                        };
                        println!("{} {} {} ({})", run.id, run.command, status, run.started_at);
                    }
                }
                RunCommands::Show { run_id } => match rfo_jobs::get_run(&conn, &run_id)? {
                    Some(run) => println!("{}", serde_json::to_string_pretty(&run)?),
                    None => eprintln!("Run {run_id} not found."),
                },
                RunCommands::Timeline { run_id } => {
                    let events = rfo_jobs::events_for_run(&conn, &run_id)?;
                    if events.is_empty() {
                        eprintln!("No events for run {run_id}.");
                    }
                    for ev in &events {
                        println!("[{}] {}: {}", ev.level, ev.ts, ev.message);
                    }
                }
            }
        }

        // ── Conflict ──
        Commands::Conflict { sub } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            match sub {
                ConflictCommands::List => {
                    let repos = manage::list(&conn, None)?;
                    let paths: Vec<PathBuf> =
                        repos.iter().map(|r| PathBuf::from(&r.local_path)).collect();
                    let conflicts = rfo_git::conflict::list_conflicts(&paths);
                    if conflicts.is_empty() {
                        eprintln!("No conflicts.");
                    }
                    for (path, state) in &conflicts {
                        let explanation = rfo_git::conflict::explain(state);
                        println!("{}: {}", path.display(), explanation);
                    }
                }
                ConflictCommands::Explain { repo } => {
                    let found = manage::find_repo(&conn, &repo)?;
                    let path = PathBuf::from(&found.local_path);
                    if let Some(state) = rfo_git::conflict::detect(&path)? {
                        println!("{}", rfo_git::conflict::explain(&state));
                    } else {
                        eprintln!("No conflict in {repo}.");
                    }
                }
                ConflictCommands::Abort { repo } => {
                    let found = manage::find_repo(&conn, &repo)?;
                    let path = PathBuf::from(&found.local_path);
                    rfo_git::conflict::abort(&path)?;
                    eprintln!("Aborted conflict in {repo}.");
                }
                ConflictCommands::MarkResolved { repo } => {
                    let found = manage::find_repo(&conn, &repo)?;
                    let path = PathBuf::from(&found.local_path);
                    match rfo_git::conflict::verify_resolved(&path) {
                        Ok(()) => {
                            eprintln!("Marked {repo} as resolved.");
                        }
                        Err(e) => {
                            anyhow::bail!("cannot mark {repo} resolved: {e}");
                        }
                    }
                }
            }
        }

        // ── Review ──
        Commands::Review { sub } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            match sub {
                ReviewCommands::Plan {
                    repo,
                    summary,
                    risk,
                } => {
                    let found = manage::find_repo(&conn, &repo)?;
                    let risk_class = match risk.as_deref() {
                        Some(r) => Some(parse_risk(r)?),
                        None => None,
                    };
                    let input = rfo_review::plan::PlanInput {
                        repo_id: Some(found.id.clone()),
                        kind: "review".into(),
                        plan_json: serde_json::json!({"summary": summary.unwrap_or_default()})
                            .to_string(),
                        risk_class,
                        risk_reasons_json: None,
                        rollback_json: None,
                    };
                    let plan = rfo_review::plan::create_plan(&conn, &input)?;
                    eprintln!("Created plan {} for {}", plan.id, repo);
                }
                ReviewCommands::Approve { plan_id } => {
                    rfo_review::plan::approve_plan(&conn, &plan_id)?;
                    eprintln!("Approved plan {plan_id}.");
                }
                ReviewCommands::Reject { plan_id } => {
                    rfo_review::plan::reject_plan(&conn, &plan_id)?;
                    eprintln!("Rejected plan {plan_id}.");
                }
                ReviewCommands::Apply { plan_id } => {
                    let result = rfo_review::apply::apply_plan(&conn, &plan_id)?;
                    eprintln!(
                        "Applied plan {} (status={}, applied_at={})",
                        result.plan_id, result.status, result.applied_at
                    );
                }
                ReviewCommands::Rollback { plan_id } => {
                    rfo_review::apply::rollback_plan(&conn, &plan_id)?;
                    eprintln!("Rolled back plan {plan_id}.");
                }
                ReviewCommands::ListPlans => {
                    let plans = rfo_review::plan::list_plans(&conn, None)?;
                    for p in &plans {
                        let repo_label = p.repo_id.as_deref().unwrap_or("-");
                        println!("{} {} {} {}", p.id, repo_label, p.kind, p.status);
                    }
                }
            }
        }

        // ── Sweep ──
        Commands::Sweep { sub } => match sub {
            SweepCommands::Commit { path, message } => {
                let result = rfo_sweep::commit::sweep_commit(&path, &message)?;
                match result {
                    rfo_sweep::commit::CommitOutcome::Committed { oid, .. } => {
                        eprintln!("Committed: {oid}");
                    }
                    rfo_sweep::commit::CommitOutcome::NothingToCommit => {
                        eprintln!("Nothing to commit.");
                    }
                    rfo_sweep::commit::CommitOutcome::BlockedByGates { failures } => {
                        eprintln!("Blocked by gates: {:?}", failures);
                    }
                    rfo_sweep::commit::CommitOutcome::BlockedBySecrets { files } => {
                        eprintln!("Blocked by secrets: {:?}", files);
                    }
                    rfo_sweep::commit::CommitOutcome::BlockedByDenylist { files } => {
                        eprintln!("Blocked by denylist: {:?}", files);
                    }
                }
            }
            SweepCommands::Agent { path, repo_id } => {
                let id = repo_id.as_deref().unwrap_or("unknown");
                let summary = rfo_sweep::agent::sweep_repo(&path, id)?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
            }
        },

        // ── Train ──
        Commands::Train { sub } => match sub {
            TrainCommands::Plan { path, repo_id } => {
                let id = repo_id.as_deref().unwrap_or("unknown");
                let plan = rfo_sweep::train::plan_train(&path, id);
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            TrainCommands::Run { path, repo_id } => {
                let id = repo_id.as_deref().unwrap_or("unknown");
                let plan = rfo_sweep::train::plan_train(&path, id);
                let result = rfo_sweep::train::run_train(&path, &plan)?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },

        // ── Doctor ──
        Commands::Doctor { fix, format } => {
            let opts = doctor::DoctorOptions {
                config_token: None,
                fix,
                binary_lookup_path: None,
                paths: Some(paths.clone()),
            };
            let report = doctor::run(opts);
            match format {
                OutputFormat::Text => println!("{}", doctor::render_text(&report)),
                OutputFormat::Json => {
                    let json =
                        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
                    println!("{json}");
                }
            }
            std::process::exit(report.exit_code());
        }
    }

    Ok(())
}
