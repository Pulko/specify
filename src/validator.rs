//! Validate a spec against the project template (dynamic contract).

use serde_yaml::Value;

pub struct CheckOutcome {
    pub ok: bool,
    pub issues: Vec<String>,
}

/// Spec must be a mapping at the root. Every key (recursively) that appears in the template must
/// be present in the spec with compatible shape. Extra keys in the spec are allowed.
pub fn validate_spec_against_template(spec: &Value, template: &Value) -> CheckOutcome {
    let mut issues = Vec::new();
    let t = peel(template);
    let s = peel(spec);

    if !t.is_mapping() {
        issues.push("template root must be a YAML mapping (object)".to_string());
        return CheckOutcome { ok: false, issues };
    }

    if !s.is_mapping() {
        issues.push("spec root must be a YAML mapping (object)".to_string());
        return CheckOutcome { ok: false, issues };
    }

    issues.extend(validate_value(s, t, ""));
    CheckOutcome {
        ok: issues.is_empty(),
        issues,
    }
}

fn peel(v: &Value) -> &Value {
    match v {
        Value::Tagged(t) => peel(&t.value),
        x => x,
    }
}

fn key_name(k: &Value) -> String {
    k.as_str()
        .map(String::from)
        .unwrap_or_else(|| format!("{k:?}"))
}

fn validate_value(spec: &Value, tmpl: &Value, path: &str) -> Vec<String> {
    let spec = peel(spec);
    let tmpl = peel(tmpl);
    let mut issues = Vec::new();

    match tmpl {
        Value::Mapping(tm) => {
            let Some(sm) = spec.as_mapping() else {
                issues.push(format!(
                    "{path}: expected mapping (object) to match template"
                ));
                return issues;
            };
            for (k, tv) in tm {
                let name = key_name(k);
                let child = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}.{name}")
                };
                let sv = sm.get(k);
                if sv.is_none() {
                    issues.push(format!("{child}: missing (required by template)"));
                    continue;
                }
                issues.extend(validate_value(sv.unwrap(), tv, &child));
            }
        }
        Value::Sequence(ts) => {
            if ts.is_empty() {
                if spec.as_sequence().is_none() {
                    issues.push(format!(
                        "{path}: expected sequence (list) to match template"
                    ));
                }
                return issues;
            }
            let Some(ss) = spec.as_sequence() else {
                issues.push(format!(
                    "{path}: expected non-empty sequence (list) to match template"
                ));
                return issues;
            };
            if ss.is_empty() {
                issues.push(format!(
                    "{path}: list must be non-empty (template defines at least one item)"
                ));
                return issues;
            }
            let proto = peel(&ts[0]);
            for (i, item) in ss.iter().enumerate() {
                let p = format!("{path}[{i}]");
                issues.extend(validate_value(item, proto, &p));
            }
        }
        Value::String(_) => {
            let Some(s) = spec.as_str() else {
                issues.push(format!("{path}: expected string to match template"));
                return issues;
            };
            if s.trim().is_empty() {
                issues.push(format!(
                    "{path}: must be a non-empty string (per template shape)"
                ));
            }
        }
        Value::Bool(_) => {
            if spec.as_bool().is_none() {
                issues.push(format!("{path}: expected boolean to match template"));
            }
        }
        Value::Number(_) => {
            if !matches!(spec, Value::Number(_)) {
                issues.push(format!("{path}: expected number to match template"));
            }
        }
        Value::Null => {
            if !spec.is_null() {
                issues.push(format!("{path}: expected null to match template"));
            }
        }
        Value::Tagged(_) => unreachable!("peel() removes tagged wrapper"),
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn spec_matches_template() {
        let tmpl = parse(
            r"
purpose: x
requirements:
  - x
scenarios:
  - name: x
    given: x
    when: x
    then: x
",
        );
        let spec = parse(
            r"
purpose: ok
requirements:
  - r1
scenarios:
  - name: n
    given: g
    when: w
    then: t
extra_field: allowed
",
        );
        let o = validate_spec_against_template(&spec, &tmpl);
        assert!(o.ok, "{:?}", o.issues);
    }

    #[test]
    fn missing_top_level_key() {
        let tmpl = parse("a: x\nb: y\n");
        let spec = parse("a: ok\n");
        let o = validate_spec_against_template(&spec, &tmpl);
        assert!(!o.ok);
        assert!(o.issues.iter().any(|i| i.contains("b")));
    }

    #[test]
    fn empty_string_fails() {
        let tmpl = parse("title: placeholder\n");
        let spec = parse("title: \"\"\n");
        let o = validate_spec_against_template(&spec, &tmpl);
        assert!(!o.ok);
    }

    #[test]
    fn empty_template_list_allows_empty_spec_list() {
        let tmpl = parse("items: []\n");
        let spec = parse("items: []\n");
        let o = validate_spec_against_template(&spec, &tmpl);
        assert!(o.ok, "{:?}", o.issues);
    }

    #[test]
    fn nested_sequence_uses_first_item_as_shape() {
        let tmpl = parse(
            r"
scenarios:
  - name: x
    step: x
",
        );
        let spec = parse(
            r"
scenarios:
  - name: a
    step: b
  - name: c
    step: d
",
        );
        let o = validate_spec_against_template(&spec, &tmpl);
        assert!(o.ok, "{:?}", o.issues);
    }
}
