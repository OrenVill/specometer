# specometer

Slice a design **spec** and superpowers-style implementation **plan** into per-task context bundles so workers only read what they need.

Design: [docs/superpowers/specs/2026-11-11-specometer-design.md](docs/superpowers/specs/2026-11-11-specometer-design.md)

## Quick install

```bash
git clone --depth 1 https://github.com/orenvill/specometer.git
cd specometer
./scripts/install.sh
export PATH="${HOME}/.local/bin:${PATH}"
```

Or without cloning:

```bash
curl -fsSL https://raw.githubusercontent.com/orenvill/specometer/master/scripts/install.sh | bash -s -- --from-git
```

This installs the `specometer` binary, then shows a **short checklist** to choose which agents get **specometer-dispatch** (Cursor, Claude Code, Codex, …). See [skills/README.md](skills/README.md).

## Build only

```bash
rustup toolchain install stable
cargo build --release
```

Binary: `target/release/specometer` (or `specometer install` from repo root)

## Usage

```bash
specometer slice --spec path/to/spec.md --plan path/to/plan.md --out bundles/
specometer bind --spec path/to/spec.md --plan path/to/plan.md --out bundles/
specometer validate --spec path/to/spec.md --plan path/to/plan.md --strict
```

Outputs:

- `bundles/task-NN-<slug>.md` — self-contained task context
- `bundles/manifest.json` — provenance, spec sections, warnings
- `bind` also writes `.specometer/manifest.json` with the same manifest (for CI drift checks)

### Spec section IDs

```markdown
## Authentication {#spec:auth}
```

or:

```markdown
## Authentication
<!-- id: spec:auth -->
```

### Plan spec refs (optional)

```markdown
**Spec refs:** spec:auth, spec:errors
```

Without refs, heuristics match headings from task title, files, and interfaces.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Parse error |
| 2 | Validation failed (`validate` or `--strict`) |
| 3 | I/O error |

## Real use (this repository)

Slice the project design spec and implementation plan:

```bash
./scripts/slice-project-docs.sh
```

Outputs under `.specometer/bundles/` (git-ignored). Point a worker at one file, e.g. `.specometer/bundles/task-04-resolver-explicit-heuristic.md`.

Override inputs:

```bash
SPEC=path/to/spec.md PLAN=path/to/plan.md OUT=./bundles ./scripts/slice-project-docs.sh
```

For production plans, add `{#spec:section-id}` on spec headings and `**Spec refs:** spec:section-id` on each task (see design spec §3).

## Orchestrator workflow

After **writing-plans** saves a plan, run `./scripts/slice-project-docs.sh` (or `specometer slice`), then dispatch subagents with **one bundle file** each under `.specometer/bundles/`.

Shipped agent skill: [skills/specometer-dispatch](skills/specometer-dispatch/SKILL.md) — install via [skills/README.md](skills/README.md). It documents bundle paths, the script, and what implementers must (and must not) read.

1. Slice (script or CLI).
2. Point each implementer at `task-NN-<slug>.md` only (see `manifest.json`).
3. Use `validate --strict` in CI when every task should declare **Spec refs**.

## Test

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
