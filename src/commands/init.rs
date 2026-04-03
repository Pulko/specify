use anyhow::{Context, Result};
use std::fs;

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

For each source file, a sibling spec may exist in the same directory: `<source_stem>` + `spec_extension` from `.specify/config.yaml` (often `.spec.yaml`).

## When reading code

1. Resolve the paired spec for the source file (basename + `spec_extension` from `.specify/config.yaml`).
2. **Prefer reading the spec** for intent, behavior, and constraints.
3. Read the **source** only if the spec is missing, insufficient, or deeper implementation detail is required.

## When modifying code

- Update the spec so it stays accurate. **Do not** let spec and code drift apart.

## When creating new source files

- Create the paired spec (run `specify generate <file>` for a template skeleton, then fill with `/spec-generate` in Cursor if needed).

## Validation

- **Structural** (YAML + shape vs `.specify/templates/<template>.yaml`): run `specify check <file>` (CLI).
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

1. Read **`.specify/config.yaml`**: note `spec_extension` and `template` (where the spec lives and which template defines the contract).
2. Read **`.specify/templates/<template>.yaml`**. That file is the **structural contract**: the spec should keep the same keys, nesting, and list shapes. Extra top-level keys in the spec are fine; anything **in** the template is required to be present and filled meaningfully.
3. Read the **source** file and open the **paired spec** next to it: same directory, `<source_stem>` + `spec_extension` from config.
4. **Replace placeholders** so the spec describes intent, constraints, and observable behavior — not implementation dumps.
5. For **list-of-object** sections, use the **sub-keys shown in the template’s first list item** for every item.

## Output

- **Write** the updated content to the existing spec path (overwrite). Keep the same YAML structure as the template defines.

## Quality bar

- Run `specify check <path-to-source>` after editing; if it fails, fix structure before finishing (check compares the spec to the template file).
"#;

const CURSOR_CMD_SPEC_CHECK: &str = r#"# /spec-check

## Preconditions

- You have both the **source file** and its **paired spec** (same path rules as the Specify CLI).

## Task

Use **`.specify/config.yaml`** and **`.specify/templates/<template>.yaml`** as the structural contract (`specify check` enforces template shape; you judge behavior).

Compare what the **spec claims** to what the **code actually does**. Report:

- **Missing behaviors** (code does something the spec does not cover)
- **Incorrect descriptions** (spec contradicts behavior)
- **Stale sections** (spec references old behavior)

## Guardrails (required)

- **Single pass:** one report per invocation; do not loop or repeatedly re-audit unless the user asks.
- **Cap:** at most **12** distinct issues; merge duplicates; prioritize by severity.
- **Scope:** only the named or selected source/spec pair — no repo-wide refactors or unrelated files.
- **Action shape:** for each issue: `location` (spec section and/or symbol), `problem` (short), `suggested_fix` (one concrete sentence or bullet).
- If a problem is **structural** (invalid YAML or fails `specify check`), say: run `specify check <file>` instead of guessing fixes.
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
    fs::write(&config_path, Config::default_yaml()).with_context(|| {
        format!("failed to write {}", config_path.display())
    })?;
    println!("wrote {}", config_path.display());

    let template_path = templates_dir.join("default.yaml");
    fs::write(&template_path, DEFAULT_TEMPLATE).with_context(|| {
        format!("failed to write {}", template_path.display())
    })?;
    println!("wrote {}", template_path.display());

    let cursor_rules = root.join(".cursor").join("rules");
    let cursor_cmds = root.join(".cursor").join("commands");
    fs::create_dir_all(&cursor_rules)?;
    fs::create_dir_all(&cursor_cmds)?;

    let rule_path = cursor_rules.join("specify.mdc");
    fs::write(&rule_path, CURSOR_RULE_SPECIFY)
        .with_context(|| format!("failed to write {}", rule_path.display()))?;
    println!("wrote {}", rule_path.display());

    let gen_path = cursor_cmds.join("spec-generate.md");
    fs::write(&gen_path, CURSOR_CMD_SPEC_GENERATE)
        .with_context(|| format!("failed to write {}", gen_path.display()))?;
    println!("wrote {}", gen_path.display());

    let chk_path = cursor_cmds.join("spec-check.md");
    fs::write(&chk_path, CURSOR_CMD_SPEC_CHECK)
        .with_context(|| format!("failed to write {}", chk_path.display()))?;
    println!("wrote {}", chk_path.display());

    Ok(())
}
