use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;
use crate::filesystem::{project_root, source_stem_from_spec_basename, SPEC_EXTENSION};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpecSyncStatus {
    InSync,
    OutOfSync,
}

#[derive(Serialize)]
struct SyncRecordJson {
    path: String,
    status: String,
    reasons: Vec<String>,
}

#[derive(Serialize)]
struct SyncJson {
    results: Vec<SyncRecordJson>,
}

pub struct SyncRecord {
    pub spec_path: PathBuf,
    pub status: SpecSyncStatus,
    pub reasons: Vec<String>,
}

pub fn run(json: bool) -> Result<bool> {
    let root = project_root();
    run_with_root(&root, json)
}

pub(crate) fn run_with_root(root: &Path, json: bool) -> Result<bool> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let config = Config::load(&root).context("run `specify init` to create .specify/config.yaml")?;

    let include = build_glob_set(&config.include)?;
    let exclude = build_glob_set(&config.exclude)?;

    let mut records = Vec::new();

    let walker = WalkDir::new(&root).into_iter().filter_entry(|e| {
        let path = e.path();
        let Some(rel) = normalize_rel(path, &root) else {
            return true;
        };
        if rel.is_empty() {
            return true;
        }
        !exclude.is_match(&rel)
    });

    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(SPEC_EXTENSION) {
            continue;
        }
        let Some(rel) = normalize_rel(path, &root) else {
            continue;
        };
        if rel.contains(".specify/") || rel.starts_with(".specify/") {
            continue;
        }

        let mut reasons = Vec::new();

        if exclude.is_match(&rel) {
            reasons.push("spec_path_excluded".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        }

        let Some(base) = source_stem_from_spec_basename(name) else {
            reasons.push("invalid_spec_basename".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        };

        let dir = path.parent().unwrap_or(Path::new("/"));
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(read_dir) = fs::read_dir(dir) {
            for ent in read_dir.flatten() {
                let p = ent.path();
                if !p.is_file() {
                    continue;
                }
                if p == path {
                    continue;
                }
                let Some(fname) = p.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if fname.ends_with(SPEC_EXTENSION) {
                    continue;
                }
                let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if stem == base {
                    candidates.push(p);
                }
            }
        }

        if candidates.is_empty() {
            reasons.push("no_matching_source".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        }

        if candidates.len() > 1 {
            reasons.push("ambiguous_source".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        }

        let source_path = &candidates[0];
        let Some(src_rel) = normalize_rel(source_path, &root) else {
            reasons.push("no_matching_source".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        };

        if exclude.is_match(&src_rel) {
            reasons.push("source_excluded".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        }

        if !include.is_match(&src_rel) {
            reasons.push("source_not_included".to_string());
            records.push(SyncRecord {
                spec_path: path.to_path_buf(),
                status: SpecSyncStatus::OutOfSync,
                reasons,
            });
            continue;
        }

        records.push(SyncRecord {
            spec_path: path.to_path_buf(),
            status: SpecSyncStatus::InSync,
            reasons: vec![],
        });
    }

    let all_ok = records
        .iter()
        .all(|r| r.status == SpecSyncStatus::InSync);

    if json {
        let results: Vec<SyncRecordJson> = records
            .iter()
            .map(|r| SyncRecordJson {
                path: r.spec_path.strip_prefix(&root).unwrap_or(&r.spec_path).to_string_lossy().replace('\\', "/"),
                status: match r.status {
                    SpecSyncStatus::InSync => "in_sync".to_string(),
                    SpecSyncStatus::OutOfSync => "out_of_sync".to_string(),
                },
                reasons: r.reasons.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string(&SyncJson { results })?);
    } else {
        for r in &records {
            let rel = r.spec_path.strip_prefix(&root).unwrap_or(&r.spec_path);
            match r.status {
                SpecSyncStatus::InSync => println!("in_sync  {}", rel.display()),
                SpecSyncStatus::OutOfSync => {
                    println!(
                        "out_of_sync  {}  [{}]",
                        rel.display(),
                        r.reasons.join(", ")
                    );
                }
            }
        }
    }

    Ok(all_ok)
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(p)?);
    }
    Ok(b.build()?)
}

fn normalize_rel(path: &Path, root: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let s = rel.to_string_lossy().replace('\\', "/");
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn write_minimal_project(root: &Path) {
        fs::create_dir_all(root.join(".specify/templates")).unwrap();
        fs::write(root.join(".specify/config.yaml"), Config::default_yaml()).unwrap();
    }

    #[test]
    fn paired_spec_in_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_minimal_project(root);
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/a.ts"), "//").unwrap();
        fs::write(root.join("lib/a.spec.yaml"), "k: v").unwrap();
        assert!(run_with_root(root, false).unwrap());
    }

    #[test]
    fn orphan_spec_out_of_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_minimal_project(root);
        fs::write(root.join("solo.spec.yaml"), "").unwrap();
        assert!(!run_with_root(root, false).unwrap());
    }

    #[test]
    fn ambiguous_source_out_of_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_minimal_project(root);
        fs::create_dir_all(root.join("x")).unwrap();
        fs::write(root.join("x/w.ts"), "").unwrap();
        fs::write(root.join("x/w.tsx"), "").unwrap();
        fs::write(root.join("x/w.spec.yaml"), "").unwrap();
        let ok = run_with_root(root, false).unwrap();
        assert!(!ok);
    }

    #[test]
    fn source_not_included() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_minimal_project(root);
        fs::create_dir_all(root.join("py")).unwrap();
        fs::write(root.join("py/m.py"), "x").unwrap();
        fs::write(root.join("py/m.spec.yaml"), "").unwrap();
        assert!(!run_with_root(root, false).unwrap());
    }
}
