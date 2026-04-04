use std::path::{Path, PathBuf};

pub fn specify_dir(root: &Path) -> PathBuf {
    root.join(".specify")
}

pub fn templates_dir(root: &Path) -> PathBuf {
    specify_dir(root).join("templates")
}

pub fn template_file(root: &Path, template_name: &str) -> PathBuf {
    templates_dir(root).join(format!("{template_name}.yaml"))
}
