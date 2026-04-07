# specify

CLI for keeping **structured YAML spec files** next to source code so coding agents (and humans) can read intent and behavior without diving into full implementations.

## Install (prebuilt binary)

The install scripts pull the **latest** [GitHub release](https://github.com/Pulko/specify/releases) for your system, check the download, and put **`specify`** somewhere you can run it from the terminal. If the command is not found right away, **open a new terminal** (on Mac/Linux the script may add one line to your shell config; Windows updates your user **PATH**).

**Optional:** set `SPECIFY_VERSION` before running the script to pin a version (for example `0.1.4` or `v0.1.4`), or `SPECIFY_INSTALL_DIR` to choose the install folder. Other knobs are documented in the script headers.

**Linux and macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Pulko/specify/main/scripts/install.sh | bash
```

**Windows (PowerShell)**:

```powershell
iwr -useb https://raw.githubusercontent.com/Pulko/specify/main/scripts/install.ps1 | iex
```

If `iex` is blocked by execution policy, run `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` once, or save the script and run it with `powershell -File install.ps1`.

You can host the same scripts on your own domain later; they still download binaries from GitHub Releases.

## Install from source (Cargo)

You need [Rust](https://rustup.rs/) (Cargo).

**Install the latest commit from a public repository:**

```bash
cargo install --git https://github.com/Pulko/specify
```

Cargo clones the repo, builds the `specify` binary, and installs it to `~/.cargo/bin` (ensure that directory is on your `PATH`).

**Install from a local clone:**

```bash
git clone https://github.com/Pulko/specify.git
cd specify
cargo install --path .
```

**Build without installing:**

```bash
cargo build --release
# binary: target/release/specify
```

### Publishing for your team

1. Push this crate to a GitHub repository.
2. Share the install script one-liners above, or `cargo install --git <url>` (optionally pin a tag: `cargo install --git <url> --tag v0.1.0`).
3. Tagged releases `v*` publish prebuilt binaries and `.sha256` checksums via GitHub Actions.

## Quick start

From your **project root** (where `.specify/` should live):

```bash
specify init                      # writes .specify/templates/default.yaml and .cursor/* (overwrites bundled files)
specify generate src/a.ts         # creates src/a.spec.yaml with root specify_template: default
specify generate src/b.rs --template other   # uses .specify/templates/other.yaml if present
specify check src/a.ts            # YAML + specify_template + template shape
specify sync                      # audit specs: paired source same stem, non-.spec.yaml (see --help)
```

Run `specify --help` and `specify <command> --help` for embedded documentation.

## Spec metadata and templates

Each spec is a YAML mapping whose root **must** include:

| Key | Meaning |
|-----|--------|
| `specify_template` | Name of the template file under `.specify/templates/` **without** `.yaml` (e.g. `default` → `default.yaml`). Put this key **first** in the file when authoring. |

`specify generate` prepends `specify_template` for you. Template files under `.specify/templates/` define **only** the structural contract (they do not repeat `specify_template`).

Add more shapes by adding `minimal.yaml`, `api.yaml`, etc., and either `specify generate --template minimal` or set `specify_template: minimal` by hand in an existing spec.

There is **no** `config.yaml`: **`specify check`** loads `.specify/templates/<specify_template>.yaml` from the spec itself.

Paired spec files are always **`<source_stem>.spec.yaml`** next to the source; the suffix is not configurable.

There is **no** separate required fields list: **`specify check`** compares each spec (minus `specify_template`) to the selected template file. Keys that appear in the template must appear in the spec with a compatible shape. Extra keys in the spec are ignored by `check`.

## `sync`

For each `*.spec.yaml`, `sync` looks in the **same directory** for exactly one other file whose name stem matches the spec (excluding other `.spec.yaml` files). That file is the paired source; extension does not matter. The walk skips `node_modules`, `target`, `.git`, and `.specify` so those trees are not scanned.

## Commands

- **`init`** — Creates/overwrites `templates/default.yaml`, Cursor rule + command stubs, and `.cursor/skills/specify/SKILL.md`.
- **`generate <file> [--template <name>]`** — Writes `<stem>.spec.yaml` with `specify_template: <name>` plus a copy of `templates/<name>.yaml` if the spec does not exist yet.
- **`check <file>`** — Spec must exist, parse as YAML, declare `specify_template`, and **match the named template structure**.
- **`sync`** — For every spec file, ensures a **unique** paired source file (same stem, not a spec). Does **not** create specs.

## Cursor

After `specify init`, use the generated rule, commands, and project skill under `.cursor/` (e.g. `/spec-generate`, `/spec-check`, and the **specify** skill for the same authoring workflow). They assume the template-driven workflow above.
