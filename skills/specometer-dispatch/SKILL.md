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

## Install specometer (once per machine / clone)

From a git clone:

```bash
./scripts/install.sh
```

That builds `specometer`, installs it to `~/.local/bin`, then shows a **checklist**: pick Cursor, Claude Code, Codex, OpenCode, Continue, Aider, and/or global Cursor (`~/.cursor/skills`). Press a number to toggle, `c` to confirm.

Without a clone:

```bash
curl -fsSL https://raw.githubusercontent.com/orenvill/specometer/master/scripts/install.sh | bash -s -- --from-git
```

CI / scripts: `specometer install --agents cursor,claude` or `./scripts/install.sh --agents cursor,claude`

Add `~/.local/bin` to `PATH`.

## Slice command (required after each plan write)

```bash
SPEC=docs/superpowers/specs/<your-spec>.md \
PLAN=docs/superpowers/plans/<your-plan>.md \
OUT=.specometer/bundles \
./scripts/slice-project-docs.sh
```

Defaults in `scripts/slice-project-docs.sh` point at this repo’s dogfood spec/plan. The script runs `validate --strict` then `slice`.

## Where outputs live

| Artifact | Path |
|----------|------|
| Per-task bundle (give this to implementers) | `$OUT/task-NN-<slug>.md` |
| Provenance and warnings | `$OUT/manifest.json` |

Resolve paths from `manifest.json` → `tasks[].bundle`.

## Orchestrator rules

1. Run the slice script once per plan revision.
2. For **Task N**, dispatch the implementer with the **single bundle file** under `$OUT`—not the full spec or plan.
3. Task reviewers may use the same bundle plus that task’s `manifest.json` entry.
4. Use explicit `**Spec refs:**` and `{#spec:...}` when possible (see root `README.md`).

## Agent coverage

`scripts/install.sh` links this skill when these project directories exist (see `skills/specometer-dispatch/agents.json`):

| Directory | Typical agent |
|-----------|----------------|
| `.cursor/skills/` | Cursor |
| `.claude/skills/` | Claude Code |
| `.codex/skills/` | OpenAI Codex |
| `.opencode/skills/` | OpenCode |
| `.continue/skills/` | Continue |
| `.aider/skills/` | Aider |

If the project has `AGENTS.md`, install appends a short specometer section once.
