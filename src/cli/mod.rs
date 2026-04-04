use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{check, generate, init, sync};

const AFTER_HELP: &str = "\
Project root is the current working directory — run commands from the repo root.

Each spec declares `specify_template` at the YAML root (first key recommended). \
Validation uses `.specify/templates/<specify_template>.yaml` as the shape contract.
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
`init` sets up .specify/templates and Cursor integration. `generate` writes a spec with `specify_template`. \
`check` validates against the named template file. `sync` audits each spec for a unique same-directory source (same stem, not .spec.yaml)."
)]
#[command(after_help = AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write `.specify/templates/default.yaml`, Cursor rule/commands, and `.cursor/skills/specify/` (overwrites)
    Init,
    /// Create spec skeleton next to source with root `specify_template` (never overwrites an existing spec)
    Generate {
        /// Source file path (relative to cwd or absolute)
        file: PathBuf,
        /// Template basename under `.specify/templates/` (file `<name>.yaml`)
        #[arg(long, default_value = "default")]
        template: String,
    },
    /// List each `.spec.yaml`: exactly one same-directory non-spec file must share its stem (no files created)
    Sync {
        /// Print JSON { results: [{ path, status, reasons }] }
        #[arg(long)]
        json: bool,
    },
    /// Valid YAML, root `specify_template`, and structure matches the named template file
    Check {
        /// Source file path (paired spec path is derived from source stem)
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
        Commands::Generate { file, template } => {
            generate::run(&file, &template)?;
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
