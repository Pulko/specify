//! Project root (cwd) and source/spec path pairing.

use std::path::{Path, PathBuf};

/// Project root is the current working directory.
pub fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// `source.ext` → `source` + spec_extension (e.g. `widget.ts` → `widget.spec.yaml`).
pub fn spec_path_for_source(source: &Path, spec_extension: &str) -> PathBuf {
    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    let stem = source.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
    dir.join(format!("{stem}{spec_extension}"))
}

/// Strip `spec_extension` from the end of the file name; returns `None` if it does not end with it.
pub fn source_stem_from_spec_basename(file_name: &str, spec_extension: &str) -> Option<String> {
    file_name.strip_suffix(spec_extension).map(str::to_string)
}

/// True if `path` (relative to project root) should be skipped entirely (e.g. `.git`, `node_modules`).
pub fn is_under_dot_specify(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".specify")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn spec_path_for_source_typescript() {
        let s = Path::new("/proj/src/widget.ts");
        let sp = spec_path_for_source(s, ".spec.yaml");
        assert_eq!(sp, Path::new("/proj/src/widget.spec.yaml"));
    }

    #[test]
    fn source_stem_from_spec_basename_ok() {
        assert_eq!(
            source_stem_from_spec_basename("widget.spec.yaml", ".spec.yaml").as_deref(),
            Some("widget")
        );
    }

    #[test]
    fn is_under_dot_specify_detects() {
        assert!(is_under_dot_specify(Path::new("a/.specify/t")));
        assert!(!is_under_dot_specify(Path::new("a/b/c")));
    }
}
