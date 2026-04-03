use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_SPEC_EXTENSION: &str = ".spec.yaml";
const DEFAULT_TEMPLATE: &str = "default";

fn default_spec_extension() -> String {
    DEFAULT_SPEC_EXTENSION.to_string()
}

fn default_template() -> String {
    DEFAULT_TEMPLATE.to_string()
}

fn default_include() -> Vec<String> {
    vec![
        "**/*.ts".to_string(),
        "**/*.tsx".to_string(),
        "**/*.js".to_string(),
        "**/*.jsx".to_string(),
    ]
}

fn default_exclude() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/target/**".to_string(),
        "**/.git/**".to_string(),
        "**/.specify/**".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_spec_extension")]
    pub spec_extension: String,
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "default_template")]
    pub template: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            spec_extension: default_spec_extension(),
            include: default_include(),
            exclude: default_exclude(),
            template: default_template(),
        }
    }
}

impl Config {
    pub fn specify_dir(root: &Path) -> PathBuf {
        root.join(".specify")
    }

    pub fn config_path(root: &Path) -> PathBuf {
        Self::specify_dir(root).join("config.yaml")
    }

    pub fn template_path(&self, root: &Path) -> PathBuf {
        Self::specify_dir(root)
            .join("templates")
            .join(format!("{}.yaml", self.template))
    }

    pub fn load(root: &Path) -> Result<Self> {
        let p = Self::config_path(root);
        let raw = std::fs::read_to_string(&p)
            .with_context(|| format!("failed to read {}", p.display()))?;
        let c: Config = serde_yaml::from_str(&raw)
            .with_context(|| format!("invalid YAML in {}", p.display()))?;
        Ok(c)
    }

    pub fn default_yaml() -> String {
        serde_yaml::to_string(&Config::default()).expect("serialize default config")
    }
}
