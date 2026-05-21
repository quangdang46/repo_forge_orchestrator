//! rfo — GitHub-first repo orchestration CLI.
//!
//! Binary entry point. CLI parsing lives here; logic in library crates and the
//! `doctor` module.
//!
//! Exit codes:
//! - `0`: healthy (or warnings only)
//! - `1`: one or more failures
//! - `64`: usage error

mod doctor;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Format::Text
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "rfo",
    about = "GitHub-first repo orchestration CLI",
    version
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Diagnose installation health.
    Doctor {
        /// Apply repairs (write default config, create state dir).
        #[arg(long)]
        fix: bool,

        /// Output format: text (default) or json.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Doctor { fix, format } => {
            let opts = doctor::DoctorOptions {
                config_token: None,
                fix,
                binary_lookup_path: None,
            };
            let report = doctor::run(opts);
            match format {
                Format::Text => println!("{}", doctor::render_text(&report)),
                Format::Json => {
                    let json = serde_json::to_string_pretty(&report)
                        .unwrap_or_else(|_| "{}".into());
                    println!("{json}");
                }
            }
            std::process::exit(report.exit_code());
        }
    }
}
