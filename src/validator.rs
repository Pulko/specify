//! Deterministic spec validation (YAML shape and required fields).

use serde_yaml::Value;

pub struct CheckOutcome {
    pub ok: bool,
    pub issues: Vec<String>,
}

pub fn validate_spec_yaml(
    yaml: &Value,
    required_fields: &[String],
) -> CheckOutcome {
    let mut issues = Vec::new();

    for key in required_fields {
        match key.as_str() {
            "purpose" => {
                if !non_empty_string(yaml.get("purpose")) {
                    issues.push("purpose: must be a non-empty string".to_string());
                }
            }
            "requirements" => {
                if !non_empty_string_list(yaml.get("requirements")) {
                    issues.push(
                        "requirements: must be a non-empty list of strings".to_string(),
                    );
                }
            }
            "scenarios" => {
                validate_scenarios(yaml.get("scenarios"), &mut issues);
            }
            other => {
                if !generic_non_empty(yaml.get(other)) {
                    issues.push(format!("{other}: required field missing or empty"));
                }
            }
        }
    }

    CheckOutcome {
        ok: issues.is_empty(),
        issues,
    }
}

fn generic_non_empty(v: Option<&Value>) -> bool {
    match v {
        None => false,
        Some(Value::Null) => false,
        Some(Value::String(s)) => !s.trim().is_empty(),
        Some(Value::Sequence(seq)) => !seq.is_empty(),
        Some(Value::Mapping(m)) => !m.is_empty(),
        Some(_) => true,
    }
}

fn non_empty_string(v: Option<&Value>) -> bool {
    match v {
        Some(Value::String(s)) => !s.trim().is_empty(),
        _ => false,
    }
}

fn non_empty_string_list(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Sequence(seq)) => {
            !seq.is_empty()
                && seq.iter().all(|item| {
                    matches!(item, Value::String(s) if !s.trim().is_empty())
                })
        }
        _ => false,
    }
}

fn validate_scenarios(v: Option<&Value>, issues: &mut Vec<String>) {
    let Some(Value::Sequence(seq)) = v else {
        issues.push("scenarios: must be a non-empty list".to_string());
        return;
    };
    if seq.is_empty() {
        issues.push("scenarios: must be a non-empty list".to_string());
        return;
    }
    for (i, item) in seq.iter().enumerate() {
        let Some(map) = item.as_mapping() else {
            issues.push(format!("scenarios[{i}]: must be a mapping"));
            continue;
        };
        for key in ["name", "given", "when", "then"] {
            let ok = mapping_str(map, key).is_some_and(|s| !s.trim().is_empty());
            if !ok {
                issues.push(format!(
                    "scenarios[{i}]: missing or empty string for `{key}`"
                ));
            }
        }
    }
}

fn mapping_str<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    map.iter()
        .find(|(k, _)| k.as_str() == Some(key))
        .and_then(|(_, v)| v.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> Vec<String> {
        vec![
            "purpose".to_string(),
            "requirements".to_string(),
            "scenarios".to_string(),
        ]
    }

    #[test]
    fn valid_minimal_spec() {
        let y = serde_yaml::from_str::<Value>(
            r"
purpose: Does something
requirements:
  - Must work
scenarios:
  - name: Happy
    given: Setup
    when: Action
    then: Outcome
",
        )
        .unwrap();
        let o = validate_spec_yaml(&y, &req());
        assert!(o.ok, "{:?}", o.issues);
    }

    #[test]
    fn missing_purpose() {
        let y = serde_yaml::from_str::<Value>(
            r"
requirements:
  - a
scenarios:
  - name: n
    given: g
    when: w
    then: t
",
        )
        .unwrap();
        let o = validate_spec_yaml(&y, &req());
        assert!(!o.ok);
    }

    #[test]
    fn empty_scenarios() {
        let y = serde_yaml::from_str::<Value>(
            r"
purpose: p
requirements:
  - r
scenarios: []
",
        )
        .unwrap();
        let o = validate_spec_yaml(&y, &req());
        assert!(!o.ok);
    }
}
