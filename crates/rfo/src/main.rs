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

    /// Sync all tracked repos
    Sync {
        /// Sync strategy: ff-only (default), rebase, merge
        #[arg(long, value_enum, default_value_t = SyncStrategy::FfOnly)]
        strategy: SyncStrategy,
        /// Max parallel jobs (default: num CPUs)
        #[arg(long)]
        jobs: Option<usize>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show status of tracked repos
    Status {
        /// Specific repo key (owner/repo, alias, or id)
        repo: Option<String>,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Prune removed/missing repos
    Prune {
        /// Skip confirmation prompts
        #[arg(long)]
        force: bool,
        /// Also prune archived repos
        #[arg(long)]
        archived: bool,
        /// Also prune missing repos (not on disk)
        #[arg(long)]
        missing: bool,
    },

    /// Diagnose installation health
    Doctor {
        /// Apply repairs (write default config, create state dir)
        #[arg(long)]
        fix: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
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

        Commands::Sync {
            strategy,
            jobs: _,
            format,
        } => {
            let conn = rfo_state::open_db(&db_path).context("opening state database")?;
            let opts = sync::SyncOptions {
                strategy,
                autostash: false,
                timeout_secs: 30,
            };
            let results = sync::sync_all(&conn, &opts).context("syncing repos")?;
            for r in &results {
                match format {
                    OutputFormat::Text => {
                        println!("{} action={} status={}", r.repo_id, r.action, r.status);
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

        Commands::Prune {
            force: _,
            archived,
            missing,
        } => {
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

        Commands::Doctor { fix, format } => {
            let opts = doctor::DoctorOptions {
                config_token: None,
                fix,
                binary_lookup_path: None,
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
