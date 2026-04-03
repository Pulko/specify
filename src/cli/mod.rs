use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{check, generate, init, sync};

const AFTER_HELP: &str = "\
Project root is the current working directory — run commands from the repo root.

Config: .specify/config.yaml  |  Template contract: .specify/templates/<template>.yaml
The `check` command validates that each spec matches the template shape (keys, nesting, lists).
Extra keys in a spec are allowed; anything present in the template must be satisfied.

Distribution: install with Cargo from a Git URL (see README.md), e.g.:
  cargo install --git https://github.com/<org>/specify
";

#[derive(Parser)]
#[command(name = "specify")]
#[command(
    version,
    about = "LLM-friendly YAML specs beside source code (template-driven validation).",
    long_about = "Specify writes and checks sibling spec files next to source. \
`init` sets up .specify/ and Cursor integration. `generate` copies the template once. \
`check` ensures the spec matches the template file. `sync` audits existing specs vs include/exclude globs."
)]
#[command(after_help = AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write .specify/config.yaml, templates/default.yaml, and Cursor rule/commands (overwrites)
    Init,
    /// Create spec skeleton from template next to source (never overwrites an existing spec)
    Generate {
        /// Source file path (relative to cwd or absolute)
        file: PathBuf,
    },
    /// List each *spec_extension file: paired source must match include/exclude (no files created)
    Sync {
        /// Print JSON { results: [{ path, status, reasons }] }
        #[arg(long)]
        json: bool,
    },
    /// Valid YAML, non-empty spec, and structure matches .specify/templates/<template>.yaml
    Check {
        /// Source file path (spec path is derived from config)
        file: PathBuf,
        /// Print JSON { ok, issues }
        #[arg(long)]
        json: bool,
    },
}

pub fn run() -> Result<i32> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            init::run()?;
            Ok(0)
        }
        Commands::Generate { file } => {
            generate::run(&file)?;
            Ok(0)
        }
        Commands::Sync { json } => {
            let ok = sync::run(json)?;
            Ok(if ok { 0 } else { 1 })
        }
        Commands::Check { file, json } => {
            let ok = check::run(&file, json)?;
            Ok(if ok { 0 } else { 1 })
        }
    }
}
