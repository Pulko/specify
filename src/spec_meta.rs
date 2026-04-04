//! Root metadata on each spec file (`specify_template`) and template file naming.

use anyhow::{bail, Result};
use serde_yaml::Value;

pub const SPECIFY_TEMPLATE_KEY: &str = "specify_template";

fn peel(v: &Value) -> &Value {
    match v {
        Value::Tagged(t) => peel(&t.value),
        x => x,
    }
}

pub fn validate_template_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("template name must not be empty");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("template name must not contain path separators or '..'");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        bail!("template name must be ASCII alphanumeric, `_`, or `-` only");
    }
    Ok(())
}

/// Reads `specify_template` from the spec root and returns the template name plus the same
/// document with that key removed for validation against `.specify/templates/<name>.yaml`.
pub fn split_spec_root(spec: &Value) -> Result<(String, Value)> {
    let v = peel(spec);
    let m = v
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("spec root must be a YAML mapping (object)"))?;
    let key = Value::String(SPECIFY_TEMPLATE_KEY.to_string());
    let raw = m.get(&key).ok_or_else(|| {
        anyhow::anyhow!(
            "spec must declare `{}` at the root (put it first in the file; picks `.specify/templates/<name>.yaml`)",
            SPECIFY_TEMPLATE_KEY
        )
    })?;
    let name = raw
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("`{}` must be a non-empty string", SPECIFY_TEMPLATE_KEY))?;
    validate_template_name(name)?;
    let mut m2 = m.clone();
    m2.remove(&key);
    Ok((name.to_string(), Value::Mapping(m2)))
}

/// New spec file contents: metadata line first, then the template body (no `specify_template` in template files).
pub fn compose_spec_file(template_name: &str, template_body: &str) -> Result<String> {
    validate_template_name(template_name)?;
    let body = template_body.trim_start();
    Ok(format!(
        "{}: {}\n\n{}",
        SPECIFY_TEMPLATE_KEY, template_name, body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_then_split_roundtrip() {
        let body = "purpose: x\n";
        let file = compose_spec_file("default", body).unwrap();
        let v: Value = serde_yaml::from_str(&file).unwrap();
        let (name, rest) = split_spec_root(&v).unwrap();
        assert_eq!(name, "default");
        let m = rest.as_mapping().unwrap();
        assert!(m.get(&Value::String("purpose".into())).is_some());
    }
}
