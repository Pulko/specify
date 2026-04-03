use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::filesystem::project_root;

const DEFAULT_TEMPLATE: &str = r#"# Spec template — customize in .specify/templates/default.yaml
purpose: "TODO: one line describing why this module exists"
requirements:
  - "TODO: replace with real requirements"
inputs: []
outputs: []
side_effects: []
dependencies: []
scenarios:
  - name: "TODO scenario name"
    given: "TODO setup context"
    when: "TODO action or trigger"
    then: "TODO expected outcome"
notes: ""
"#;

const CURSOR_RULE_SPECIFY: &str = r#"---
description: Spec-first reading and Specify workflow
alwaysApply: true
---

# Specify (spec files)

For each source file, a sibling spec may exist: same basename with `.spec.yaml` (or the project `spec_extension` from `.specify/config.yaml`).

## When reading code

1. Resolve the paired spec for the source file (basename + `spec_extension` from `.specify/config.yaml`).
2. **Prefer reading the spec** for intent, behavior, and constraints.
3. Read the **source** only if the spec is missing, insufficient, or deeper implementation detail is required.

## When modifying code

- Update the spec so it stays accurate. **Do not** let spec and code drift apart.

## When creating new source files

- Create the paired spec (run `specify generate <file>` for a template skeleton, then fill with `/spec-generate` in Cursor if needed).

## Validation

- **Structural** (YAML, required fields): run `specify check <file>` (CLI).
- **Semantic** (spec vs actual code behavior): use the `/spec-check` Cursor command after substantive changes.
"#;

const CURSOR_CMD_SPEC_GENERATE: &str = r#"# /spec-generate

## Preconditions

- The **spec file must already exist** (skeleton). If it is missing, run:

  `specify generate <path-to-source-file>`

  then continue.

## Inputs

- **Source file** (the implementation) the user points to or has open.

## Task

1. Read **`.specify/config.yaml`**: note `spec_extension`, `template`, and `required_fields` (these define where the spec lives and what the CLI treats as mandatory).
2. Read the active template file **`.specify/templates/<template>.yaml`** (from the `template` key). That file is the **schema contract** for this repo: same keys, nesting, and list structures. Do not invent a different shape unless the user explicitly asked to change the template.
3. Read the **source** file and open the **paired spec** next to it: same directory, basename `<source_stem>` + `spec_extension` from config (not always `.spec.yaml`).
4. **Replace placeholders and empty sections** in the spec so they describe intent, constraints, and observable behavior — aligned with the template’s fields and lists (whatever they are named).
5. For any **scenario-style** blocks the template defines, write concrete, testable scenarios using **the sub-keys that template uses** (do not assume fixed names unless the template shows them).

## Output

- **Write** the updated content to the existing spec path (overwrite). Preserve the project’s YAML structure as defined by the template.

## Quality bar

- Concise, structured, and consistent with **this repo’s** template and `required_fields`.
- Run `specify check <path-to-source>` after editing; if it fails, fix structure before finishing (the check honors `required_fields` from config).
"#;

const CURSOR_CMD_SPEC_CHECK: &str = r#"# /spec-check

## Preconditions

- You have both the **source file** and its **paired spec** (same path rules as the Specify CLI).

## Task

Use **`.specify/config.yaml`** (`template`, `required_fields`, `spec_extension`) and **`.specify/templates/<template>.yaml`** as the project’s declared spec contract when judging completeness.

Compare what the **spec claims** to what the **code actually does**. Report:

- **Missing behaviors** (code does something the spec does not cover)
- **Incorrect descriptions** (spec contradicts behavior)
- **Stale sections** (spec references old behavior)

## Guardrails (required)

- **Single pass:** one report per invocation; do not loop or repeatedly re-audit unless the user asks.
- **Cap:** at most **12** distinct issues; merge duplicates; prioritize by severity.
- **Scope:** only the named or selected source/spec pair — no repo-wide refactors or unrelated files.
- **Action shape:** for each issue: `location` (spec section and/or symbol), `problem` (short), `suggested_fix` (one concrete sentence or bullet).
- If a problem is **structural** (invalid YAML, missing required fields), say: run `specify check <file>` instead of guessing fixes.
- **Stop** after the list. **Do not** apply edits unless the user explicitly asks you to fix them.

## Output format

Numbered list of issues with the fields above. No preamble essay.
"#;

pub fn run() -> Result<()> {
    let root = project_root();
    let specify_dir = Config::specify_dir(&root);
    let templates_dir = specify_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    let config_path = Config::config_path(&root);
    if !config_path.exists() {
        fs::write(&config_path, Config::default_yaml()).with_context(|| {
            format!("failed to write {}", config_path.display())
        })?;
        println!("created {}", config_path.display());
    } else {
        println!("exists (skipped): {}", config_path.display());
    }

    let template_path = templates_dir.join("default.yaml");
    if !template_path.exists() {
        fs::write(&template_path, DEFAULT_TEMPLATE).with_context(|| {
            format!("failed to write {}", template_path.display())
        })?;
        println!("created {}", template_path.display());
    } else {
        println!("exists (skipped): {}", template_path.display());
    }

    let cursor_rules = root.join(".cursor").join("rules");
    let cursor_cmds = root.join(".cursor").join("commands");
    fs::create_dir_all(&cursor_rules)?;
    fs::create_dir_all(&cursor_cmds)?;

    write_if_missing(&cursor_rules.join("specify.mdc"), CURSOR_RULE_SPECIFY)?;
    write_if_missing(
        &cursor_cmds.join("spec-generate.md"),
        CURSOR_CMD_SPEC_GENERATE,
    )?;
    write_if_missing(&cursor_cmds.join("spec-check.md"), CURSOR_CMD_SPEC_CHECK)?;

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        println!("exists (skipped): {}", path.display());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    println!("created {}", path.display());
    Ok(())
}
