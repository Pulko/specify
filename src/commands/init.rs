use anyhow::{Context, Result};
use std::fs;

use crate::filesystem::project_root;
use crate::paths::templates_dir;

const DEFAULT_TEMPLATE: &str = r#"# Spec template — customize in .specify/templates/default.yaml
purpose: "TODO: one line describing why this module exists"
requirements:
  - "TODO: replace with real requirements"
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

- If a **paired spec exists** for a source file you change, **update that spec in the same turn** (or before you stop) so it matches the code: observable behavior, public API, errors, and any other claims your template captures. **Do not** leave the paired spec stale after behavior-affecting edits unless the user explicitly asked to touch code only.
- If no paired spec exists yet, follow **When the spec is missing** when adding or substantially editing that source.

## Validation

- **Structure:** `specify check <path-to-source>` (spec root must declare `specify_template`; shape vs `.specify/templates/<name>.yaml`).
- **Accuracy:** `/spec-check` after substantive changes.
"#;

/// Shared body for `/spec-generate` and the Cursor skill (`concat!` requires literals).
macro_rules! spec_generate_procedure {
    () => {
        r#"## Preconditions

- The **spec file must already exist** (skeleton). If it is missing, run:

  `specify generate <path-to-source-file>`

  then continue.

## Inputs

- **Source file** (the implementation) the user points to or has open.

## Task

1. Read the **paired spec** first: same directory as the source, `<source_stem>.spec.yaml`. At the root, read **`specify_template`** (non-empty string; put it **first** in the file when authoring). That names **`.specify/templates/<specify_template>.yaml`**.
2. Read **`.specify/templates/<specify_template>.yaml`** (structural contract: keys, nesting, list shapes). Template files do **not** contain `specify_template`.
3. Read the **source** file only after the spec (and template) are loaded.
4. **Replace placeholders** so the spec describes intent, constraints, and observable behavior — not implementation dumps. **Preserve** `specify_template` and keep it accurate if the spec should use a different template file.
5. For **list-of-object** sections, use the **sub-keys shown in the template’s first list item** for every item.

## Output

- **Write** the updated content to the existing spec path (overwrite). Keep the same YAML structure as the template defines.

## Quality bar

- Run `specify check <path-to-source>` after editing; if it fails, fix structure before finishing (check uses `specify_template` to pick the template file).
"#
    };
}

const CURSOR_CMD_SPEC_GENERATE: &str = concat!("# /spec-generate\n\n", spec_generate_procedure!());

const CURSOR_SKILL_SPECIFY: &str = concat!(
    r#"---
name: specify
description: >-
  Use when creating, filling in, or refreshing paired `.spec.yaml` files; when code was
  edited and a paired spec exists for that source path (eliminate drift — update the spec
  in the same task); or when aligning specs with `.specify/templates/`. For Specify projects
  (`dir/name.spec.yaml` beside `dir/name.ext`).
---

# Specify — authoring paired specs

This project keeps **LLM-oriented YAML specs** beside sources: for `dir/name.ext`, the paired spec is **`dir/name.spec.yaml`**.

**Project rule** (if present): read an existing paired spec **before** the source when explaining or changing behavior. This skill focuses on **writing or updating** that spec from the template contract.

## No spec drift (mandatory)

When you **change** a source file (edits, refactors, new behavior, API changes) and its paired **`<stem>.spec.yaml` already exists**:

1. **Update the spec in the same task** before you finish — treating it as part of the change, not optional follow-up.
2. Bring **purpose, requirements, scenarios, inputs/outputs, side effects, dependencies**, and any other template sections in line with **what the code does now** (observable behavior and public contracts — not a line-by-line mirror of implementation).
3. Run **`specify check <path-to-source>`** after editing the spec; fix structural mismatches before stopping.

**Do not** stop with a stale paired spec after behavior-affecting code edits unless the user **explicitly** asked to skip spec updates.

**CLI:** `specify generate <path-to-source>` (optional `--template <name>`, default `default`) creates the skeleton when the spec is missing; then use the procedure below (same as `/spec-generate`).

"#,
    spec_generate_procedure!(),
    r#"

## After substantive edits

- Re-read spec vs code yourself or use **`/spec-check`** for a focused audit; if you find drift, **edit the spec** (or the code, if the spec was correct) — reporting alone is not enough when you are the one who introduced the change.

## Slash commands (equivalent entry points)

- **`/spec-generate`** — same procedure as above.
- **`/spec-check`** — audit spec vs implementation without applying fixes unless asked.
"#
);

const CURSOR_CMD_SPEC_CHECK: &str = r#"# /spec-check

## Preconditions

- You have both the **source file** and its **paired spec** (same path rules as the Specify CLI).

## Task

1. Read the **paired spec** (`<source_stem>.spec.yaml` next to the source) in full **before** re-reading the source — note root **`specify_template`** (which `.specify/templates/<name>.yaml` defines structure).
2. Read **`.specify/templates/<specify_template>.yaml`** as the structural contract (`specify check` enforces shape; you judge behavior).
3. Read the **source** and compare to the spec.

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
    let templates = templates_dir(&root);
    fs::create_dir_all(&templates)?;

    let template_path = templates.join("default.yaml");
    fs::write(&template_path, DEFAULT_TEMPLATE)
        .with_context(|| format!("failed to write {}", template_path.display()))?;
    println!("wrote {}", template_path.display());

    let cursor_rules = root.join(".cursor").join("rules");
    let cursor_cmds = root.join(".cursor").join("commands");
    let cursor_skill_dir = root.join(".cursor").join("skills").join("specify");
    fs::create_dir_all(&cursor_rules)?;
    fs::create_dir_all(&cursor_cmds)?;
    fs::create_dir_all(&cursor_skill_dir)?;

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

    let skill_path = cursor_skill_dir.join("SKILL.md");
    fs::write(&skill_path, CURSOR_SKILL_SPECIFY)
        .with_context(|| format!("failed to write {}", skill_path.display()))?;
    println!("wrote {}", skill_path.display());

    Ok(())
}
