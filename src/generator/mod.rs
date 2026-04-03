use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;

pub fn read_template(root: &Path, config: &Config) -> Result<String> {
    let p = config.template_path(root);
    fs::read_to_string(&p).with_context(|| format!("failed to read template {}", p.display()))
}

pub fn write_spec_skeleton(spec_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(spec_path, content).with_context(|| format!("failed to write {}", spec_path.display()))?;
    Ok(())
}
