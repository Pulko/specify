use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::filesystem::{project_root, spec_path_for_source};
use crate::paths::template_file;
use crate::spec_meta::split_spec_root;
use crate::validator::validate_spec_against_template;

#[derive(Serialize)]
struct CheckJson {
    ok: bool,
    issues: Vec<String>,
}

pub fn run(source_arg: &Path, json: bool) -> Result<bool> {
    let root = project_root();

    let source = resolve_source_path(&root, source_arg)?;
    if !source.is_file() {
        bail!("not a file: {}", source.display());
    }

    let spec_path = spec_path_for_source(&source);
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

    let (template_name, spec_body) = match split_spec_root(&value) {
        Ok(pair) => pair,
        Err(e) => {
            issues.push(e.to_string());
            return finish(json, false, issues);
        }
    };

    let template_path = template_file(&root, &template_name);
    let template_raw = match fs::read_to_string(&template_path) {
        Ok(s) => s,
        Err(e) => {
            issues.push(format!(
                "cannot read template `{}` ({}): {e}",
                template_path.display(),
                template_name
            ));
            return finish(json, false, issues);
        }
    };

    let template: serde_yaml::Value = match serde_yaml::from_str(&template_raw) {
        Ok(v) => v,
        Err(e) => {
            issues.push(format!("invalid template YAML ({}): {e}", template_name));
            return finish(json, false, issues);
        }
    };

    let outcome = validate_spec_against_template(&spec_body, &template);
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
