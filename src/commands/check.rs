use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::filesystem::{project_root, spec_path_for_source};
use crate::validator::validate_spec_yaml;

#[derive(Serialize)]
struct CheckJson {
    ok: bool,
    issues: Vec<String>,
}

pub fn run(source_arg: &Path, json: bool) -> Result<bool> {
    let root = project_root();
    let config = Config::load(&root).context("run `specify init` to create .specify/config.yaml")?;

    let source = resolve_source_path(&root, source_arg)?;
    if !source.is_file() {
        bail!("not a file: {}", source.display());
    }

    let spec_path = spec_path_for_source(&source, &config.spec_extension);
    let mut issues = Vec::new();

    if !spec_path.is_file() {
        issues.push(format!(
            "spec file missing (expected {})",
            spec_path.display()
        ));
        return finish(json, false, issues);
    }

    let raw = match fs::read_to_string(&spec_path) {
        Ok(s) => s,
        Err(e) => {
            issues.push(format!("cannot read spec: {e}"));
            return finish(json, false, issues);
        }
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        issues.push("spec file is empty".to_string());
        return finish(json, false, issues);
    }

    let value: serde_yaml::Value = match serde_yaml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            issues.push(format!("invalid YAML: {e}"));
            return finish(json, false, issues);
        }
    };

    let outcome = validate_spec_yaml(&value, &config.required_fields);
    issues.extend(outcome.issues);

    let ok = issues.is_empty();
    finish(json, ok, issues)
}

fn finish(json: bool, ok: bool, issues: Vec<String>) -> Result<bool> {
    if json {
        let out = CheckJson { ok, issues };
        println!("{}", serde_json::to_string(&out)?);
    } else if ok {
        println!("check passed");
    } else {
        for line in &issues {
            println!("{line}");
        }
    }
    Ok(ok)
}

fn resolve_source_path(root: &Path, arg: &Path) -> Result<PathBuf> {
    let p = if arg.is_absolute() {
        arg.to_path_buf()
    } else {
        root.join(arg)
    };
    Ok(fs::canonicalize(&p).unwrap_or(p))
}
