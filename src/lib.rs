pub mod cli;
pub mod commands;
pub mod filesystem;
pub mod generator;
pub mod paths;
pub mod spec_meta;
pub mod validator;

use anyhow::Result;

pub fn run() -> Result<i32> {
    cli::run()
}
