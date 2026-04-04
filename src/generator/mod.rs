use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::paths::template_file;
use crate::spec_meta::{compose_spec_file, validate_template_name};

pub fn read_template(root: &Path, template_name: &str) -> Result<String> {
    validate_template_name(template_name)?;
    let p = template_file(root, template_name);
    fs::read_to_string(&p).with_context(|| format!("failed to read template {}", p.display()))
}

pub fn write_spec_skeleton(
    spec_path: &Path,
    template_name: &str,
    template_body: &str,
) -> Result<()> {
    let content = compose_spec_file(template_name, template_body)?;
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(spec_path, content)
        .with_context(|| format!("failed to write {}", spec_path.display()))?;
    Ok(())
}
