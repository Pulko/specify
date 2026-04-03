pub mod cli;
pub mod commands;
pub mod config;
pub mod filesystem;
pub mod generator;
pub mod validator;

use anyhow::Result;

pub fn run() -> Result<i32> {
    cli::run()
}
