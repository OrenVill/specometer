---
name: specometer-dispatch
description: >-
  After an implementation plan is written and saved, runs specometer to slice
  spec and plan into per-task context bundles and directs orchestrators to give
  subagents only the bundle path. Use when finishing writing-plans, before
  subagent-driven-development or executing-plans, or when dispatching per-task
  implementer subagents.
---

# Specometer dispatch

Specometer turns a **design spec** + **implementation plan** into one markdown file per plan task. Implementer subagents must receive **only** that bundle—not the full spec or plan.

## When to run

Run this workflow **immediately after** the implementation plan is saved (end of **writing-plans**), **before** the first implementer subagent is dispatched (**subagent-driven-development** or **executing-plans**).

If the spec or plan changed, run the script again and dispatch against the new bundles.

## Slice command (required)

From the repository root (after `cargo build --release` once, or let the script build):

```bash
SPEC=docs/superpowers/specs/<your-spec>.md \
PLAN=docs/superpowers/plans/<your-plan>.md \
OUT=.specometer/bundles \
./scripts/slice-project-docs.sh
```

Defaults inside the script (this repo’s dogfood paths):

- `SPEC` → `docs/superpowers/specs/2026-11-11-specometer-design.md`
- `PLAN` → `docs/superpowers/plans/2026-11-11-specometer.md`
- `OUT` → `.specometer/bundles`

The script runs `specometer validate --strict`, then `specometer slice`. Fix spec refs or section IDs if validation fails.

## Where outputs live

| Artifact | Path |
|----------|------|
| Per-task bundle (give this to implementers) | `$OUT/task-NN-<slug>.md` (e.g. `.specometer/bundles/task-04-resolver-explicit-heuristic.md`) |
| Provenance and warnings | `$OUT/manifest.json` |
| Optional CI bind copy | `.specometer/manifest.json` (only when using `specometer bind`, not the default script) |

Task order and slugs come from `manifest.json` → `tasks[].slug` and `tasks[].bundle`.

## Orchestrator rules

1. Run the slice script once per plan revision.
2. For **Task N**, dispatch the implementer with:
   - Repository root as working directory
   - **Single context file:** absolute or repo-relative path to `$OUT/<bundle>` from the manifest
   - Do **not** attach the full spec, full plan, or prior chat unless the manifest lists warnings for that task.
3. Task reviewers may use the same bundle plus that task’s entry in `manifest.json`.
4. Prefer explicit `**Spec refs:**` on each plan task and `{#spec:...}` on spec headings (see repo `README.md`).

## Install this skill

Copy into your skills directory so the agent can load it:

```bash
mkdir -p ~/.cursor/skills
cp -r skills/specometer-dispatch ~/.cursor/skills/specometer-dispatch
```

Or symlink from a project (not committed here): `.cursor/skills/specometer-dispatch` → `skills/specometer-dispatch`.

## Build specometer

```bash
cargo build --release
# binary: target/release/specometer
```
