# specify

CLI for keeping **structured YAML spec files** next to source code so coding agents (and humans) can read intent and behavior without diving into full implementations.

## Install from GitHub

You need [Rust](https://rustup.rs/) (Cargo).

**Install the latest commit from a public repository:**

```bash
cargo install --git https://github.com/Pulko/specify
```

Replace `Pulko/specify` with your real GitHub path. Cargo clones the repo, builds the `specify` binary, and installs it to `~/.cargo/bin` (ensure that directory is on your `PATH`).

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
2. Share the `cargo install --git <url>` command (optionally pin a tag: `cargo install --git <url> --tag v0.1.0`).
3. Optional: add **GitHub Releases** with prebuilt binaries via a workflow (not included here); many teams rely on `cargo install --git` only.

## Quick start

From your **project root** (where `.specify/` should live):

```bash
specify init              # writes .specify/* and .cursor/* (overwrites bundled files)
specify generate src/a.ts # creates src/a.spec.yaml from the template if missing
specify check src/a.ts    # YAML + template shape
specify sync              # audit existing specs vs include/exclude (see --help)
```

Run `specify --help` and `specify <command> --help` for embedded documentation.

## Configuration (`.specify/config.yaml`)

| Field | Meaning |
|--------|---------|
| `template` | Name of template file under `.specify/templates/` (default `default` → `default.yaml`) |
| `include` | Glob patterns for tracked sources (used by `sync`) |
| `exclude` | Glob patterns to skip (used by `sync` and directory walking) |

Paired spec files are always **`<source_stem>.spec.yaml`** next to the source; the suffix is not configurable. If an older `config.yaml` still has a `spec_extension` key, it is ignored.

There is **no** separate required fields list: **`specify check` compares each spec to your template file**. Keys that appear in the template must appear in the spec with a compatible shape. Extra keys in the spec are ignored by `check`.

Edit `.specify/templates/<template>.yaml` to change the contract for new `generate` output and for `check` validation.

## Commands

- **`init`** — Creates/overwrites default config, default template, and Cursor rule + command stubs.
- **`generate <file>`** — Copies the template to `<stem>.spec.yaml` if it does not exist yet.
- **`check <file>`** — Spec must exist, parse as YAML, be non-empty, and **match the template structure**.
- **`sync`** — For every existing spec file, ensures a unique paired source exists and matches `include` / `exclude`. Does **not** create specs.

## Cursor

After `specify init`, use the generated rule and commands under `.cursor/` (e.g. `/spec-generate`, `/spec-check`). They assume the template-driven workflow above.

## License

Specify your license in `Cargo.toml` and add a `LICENSE` file when you publish.
