use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{check, generate, init, sync};

#[derive(Parser)]
#[command(name = "specify")]
#[command(about = "Generate and maintain LLM-friendly spec files beside source code.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .specify/, default config and template, and Cursor rule/commands
    Init,
    /// Create a spec skeleton next to a source file from the template (does not overwrite)
    Generate {
        /// Path to the source file (relative to current directory or absolute)
        file: PathBuf,
    },
    /// Report whether each existing spec is in sync with config (include/exclude and paired source)
    Sync {
        /// Emit JSON report
        #[arg(long)]
        json: bool,
    },
    /// Validate spec YAML and required fields for a source file
    Check {
        /// Path to the source file
        file: PathBuf,
        /// Emit JSON result
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
