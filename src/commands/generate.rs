use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::filesystem::{project_root, spec_path_for_source};
use crate::generator::{read_template, write_spec_skeleton};

pub fn run(source_arg: &Path) -> Result<()> {
    let root = project_root();
    let config = Config::load(&root).context("run `specify init` to create .specify/config.yaml")?;

    let source = resolve_source_path(&root, source_arg)?;
    if !source.is_file() {
        bail!("not a file: {}", source.display());
    }

    let spec_path = spec_path_for_source(&source, &config.spec_extension);
    if spec_path.exists() {
        println!("spec already exists: {}", spec_path.display());
        return Ok(());
    }

    let template = read_template(&root, &config)?;
    write_spec_skeleton(&spec_path, &template)?;
    println!("wrote {}", spec_path.display());
    Ok(())
}

fn resolve_source_path(root: &Path, arg: &Path) -> Result<PathBuf> {
    let p = if arg.is_absolute() {
        arg.to_path_buf()
    } else {
        root.join(arg)
    };
    Ok(fs::canonicalize(&p).unwrap_or(p))
}
