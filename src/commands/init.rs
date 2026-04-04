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
description: Mandatory spec-before-source when a paired spec exists (Specify)
alwaysApply: true
---

# Specify — read paired specs first

This project keeps **LLM-oriented YAML specs** next to sources. Your default workflow is **textual context first**, implementation second.

## Paired spec path

For a source file `dir/name.ext`, the paired spec is always **`dir/name.spec.yaml`** (same directory, source stem + `.spec.yaml`).

## Mandatory read order (when the spec file exists)

If that paired spec path **exists on disk**:

1. **Read the spec first** in this turn — before reading the paired source for intent, behavior, API contracts, or “what does this do?”
2. **Then** read the source only for details not stated in the spec, verification, or edits.

Do **not** treat the spec as optional flavor text. If it exists, it is the **primary** context for that module unless the user explicitly asks to ignore it or only wants raw code.

## Triggers — apply the order above when

- The user @-mentions, opens, or names a **source** path that can have a paired spec, **or**
- You would otherwise read implementation files to explain, review, or change behavior.

**First action:** resolve the paired spec path; **if the file exists, read it before the source.**

## When the spec is missing

- Read the source as usual. For new files, create the paired spec (`specify generate <file>` then `/spec-generate` if helpful).

## When modifying code

- Keep spec and behavior aligned; update the spec when you change observable behavior or public API.

## Validation

- **Structure:** `specify check <path-to-source>` (vs `.specify/templates/<template>.yaml`).
- **Accuracy:** `/spec-check` after substantive changes.
"#;

const CURSOR_CMD_SPEC_GENERATE: &str = r#"# /spec-generate

## Preconditions

- The **spec file must already exist** (skeleton). If it is missing, run:

  `specify generate <path-to-source-file>`

  then continue.

## Inputs

- **Source file** (the implementation) the user points to or has open.

## Task

1. Read **`.specify/config.yaml`** for **`template`** only (which file under `.specify/templates/` defines the contract).
2. Read **`.specify/templates/<template>.yaml`** (structural contract: keys, nesting, list shapes).
3. Read the **paired spec** first: same directory as the source, `<source_stem>.spec.yaml`.
4. Read the **source** file only after the spec (and template) are loaded.
5. **Replace placeholders** so the spec describes intent, constraints, and observable behavior — not implementation dumps.
6. For **list-of-object** sections, use the **sub-keys shown in the template’s first list item** for every item.

## Output

- **Write** the updated content to the existing spec path (overwrite). Keep the same YAML structure as the template defines.

## Quality bar

- Run `specify check <path-to-source>` after editing; if it fails, fix structure before finishing (check compares the spec to the template file).
"#;

const CURSOR_CMD_SPEC_CHECK: &str = r#"# /spec-check

## Preconditions

- You have both the **source file** and its **paired spec** (same path rules as the Specify CLI).

## Task

1. Read **`.specify/config.yaml`** for **`template`** (template file name under `.specify/templates/`).
2. Read the **paired spec** (`<source_stem>.spec.yaml` next to the source) in full **before** re-reading the source — anchor your expectations in the spec first.
3. Read the **source** and compare to the spec.

Use **`.specify/templates/<template>.yaml`** as the structural contract (`specify check` enforces shape; you judge behavior).

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
