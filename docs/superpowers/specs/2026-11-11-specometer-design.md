# Specometer — Task Context Slicing

**Status:** Draft (design approved in brainstorming)  
**Date:** 2026-11-11

## 1. Purpose and boundaries

**Specometer** is a standalone CLI and library in this repository. It reads a design spec and an implementation plan (superpowers-style markdown) and emits one **task context bundle** per plan task. A subagent (or any worker) receives only that bundle and does not need to open or search the full spec or plan.

### In scope (v1)

- Parse plan structure: `### Task N:` blocks, Global Constraints, Files, Interfaces (Consumes / Produces), checkbox steps
- Parse spec structure: heading hierarchy plus optional stable **section IDs**
- Resolve spec slices per task: **explicit refs first**, then **heuristic fallback**
- Include plan dependency glue: Global Constraints, full current task block, and prior-task interface outputs required as consumes (closure within the plan only)
- Emit human-readable bundle files and a machine-readable manifest (provenance, confidence, warnings)

### Out of scope (v1)

- Cursor-specific skills, paths under `cursor/`, or co-authored commits
- LLM-based slicing (reserved extension point)
- Dispatching or running subagents
- Editing spec/plan source files (read-only inputs)

### Non-goals

- Replace superpowers `writing-plans` or `subagent-driven-development`; specometer complements them by automating context assembly
- Guarantee perfect semantic relevance without explicit refs (heuristics are best-effort with visible warnings)

## 2. Architecture (Approach 2 + optional bind)

```
┌─────────────┐     ┌─────────────┐
│  spec.md    │     │  plan.md    │
└──────┬──────┘     └──────┬──────┘
       │                   │
       └────────┬──────────┘
                ▼
       ┌────────────────┐
       │  specometer-   │
       │  core (Rust)   │
       │  - SpecIndex   │
       │  - PlanIndex   │
       │  - Resolver    │
       │  - Bundler     │
       └────────┬───────┘
                ▼
       ┌────────────────┐
       │  CLI           │
       │  slice | bind  │
       │  validate      │
       └────────┬───────┘
                ▼
    out/task-01-*.md  +  manifest.json
```

| Package | Responsibility |
|---------|----------------|
| `@specometer/core` | Markdown AST, indexes, resolution, bundle assembly |
| `specometer` (CLI) | Argument parsing, I/O paths, exit codes, stdout summaries |

**Commands (v1)**

| Command | Behavior |
|---------|----------|
| `specometer slice --spec PATH --plan PATH --out DIR` | Default on-demand generation |
| `specometer bind` | Same as `slice` plus write `.specometer/manifest.json` with content hashes of inputs (for CI drift checks) |
| `specometer validate --spec PATH --plan PATH` | No write; exit non-zero on parse errors, missing explicit refs, or empty heuristic matches |

Optional **bind** mode (Approach 3) is opt-in so authors are not forced to re-bind on every typo; CI can require `bind` + committed artifacts when the team wants that gate.

## 3. Input conventions

### Spec section IDs (optional, preferred)

Pandoc-style heading attributes:

```markdown
## Authentication {#spec:auth}
```

Or an HTML comment on the line immediately after the heading:

```markdown
## Authentication
<!-- id: spec:auth -->
```

IDs must match `^spec:[a-z0-9][a-z0-9-]*$`.

### Plan explicit refs (optional)

Inside a task block:

```markdown
**Spec refs:** spec:auth, spec:errors
```

Refs are comma-separated; whitespace is trimmed. Unknown IDs produce `validate` errors and `slice` warnings (configurable strict mode).

### Plan task identity

- Task number and title from `### Task N: Title`
- Stable slug for filenames: `task-NN-kebab-title` (NN zero-padded; slug from title, ASCII, max 48 chars)

### Superpowers plan header

Bundler always copies **Global Constraints** (verbatim from plan) into every task bundle.

## 4. Resolution algorithm

Per task, build the set of spec sections `S`:

### Step A — Explicit

For each ID in `**Spec refs:**`, add the section with that ID (including nested content until the next same-or-higher-level heading).

### Step B — Heuristic fallback

If `S` is empty or `--heuristic=always` (default: `when-no-explicit`):

1. Tokenize: task title words, file basenames/dirnames from **Files**, symbol names from **Interfaces**
2. Score each spec section (heading + first paragraph): overlap with tokens; boost if heading slug appears in a file path
3. Take top-K sections (default K=3) above a minimum score threshold
4. Add sections whose headings appear as substantial phrases in **Global Constraints** (normalized match)

### Step C — Plan cross-task closure

1. Parse **Consumes** for types/symbols/names
2. Walk earlier tasks in plan order; for each earlier **Produces** entry that satisfies a consume, append that task’s **Interfaces** block (not the full earlier task) to the bundle under `## Prior task interfaces`
3. Do not include later tasks

### Limits and warnings

| Limit | Default | Manifest |
|-------|---------|----------|
| Max heuristic sections | 5 | `truncated: true` |
| Max lines per spec section | 200 | `section_truncated: ["spec:auth"]` |
| No matches | — | `heuristic_empty: true`, severity warning |

Each included spec section is tagged in manifest as `explicit` or `heuristic` with optional `score`.

## 5. Bundle output schema

### File: `out/task-NN-<slug>.md`

Fixed section order:

1. **YAML front matter** (minimal): `task`, `spec`, `plan`, `generated_at`, `specometer_version`
2. **Global Constraints** (from plan)
3. **This task** (full `### Task N` block, unchanged)
4. **Spec context** (only selected sections, each with a source line comment: `<!-- from: spec:auth lines 42-98 -->`)
5. **Prior task interfaces** (if any)
6. **Dispatch note** (static boilerplate): bundle is self-contained; do not re-read full spec/plan unless manifest lists warnings

### File: `out/manifest.json`

```json
{
  "spec": { "path": "...", "sha256": "..." },
  "plan": { "path": "...", "sha256": "..." },
  "tasks": [
    {
      "id": "task-3",
      "slug": "task-03-login-api",
      "bundle": "task-03-login-api.md",
      "specSections": [
        { "id": "spec:auth", "mode": "explicit", "lines": [42, 98] },
        { "id": null, "mode": "heuristic", "heading": "Error handling", "score": 0.72, "lines": [120, 145] }
      ],
      "priorTasks": ["task-1"],
      "warnings": []
    }
  ]
}
```

With `bind`, duplicate manifest under `.specometer/manifest.json` and record input hashes for CI comparison.

## 6. Tech stack

**Language: Rust (edition 2021, MSRV 1.78+)** — chosen for memory safety without a GC, predictable performance on large specs, and a single static binary with no Node/npm supply-chain surface for a tool that shapes agent context.

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Core crate | `specometer-core` | Library: parse, resolve, bundle; `#![forbid(unsafe_code)]` |
| CLI crate | `specometer-cli` | Thin `clap` front-end over core; binary name `specometer` |
| Markdown | `pulldown-cmark` (+ `pulldown-cmark-to-cmark` only if round-trip needed) | Fast; heading events with source offsets for line ranges |
| JSON manifest | `serde` + `serde_json` | Strict schema; deny unknown fields on read where applicable |
| Hashing | `sha2` | Input checksums for `bind` / CI |
| Tests | `cargo test`, `insta` snapshots | Unit + integration; fixtures as golden files |
| Layout | Cargo workspace: `crates/specometer-core`, `crates/specometer-cli` | Core testable without spawning the binary |

**Security posture (v1)**

- No network I/O in the CLI (filesystem in, filesystem out only).
- Avoid `unwrap()` on user-controlled paths; propagate errors with context (`anyhow` / `thiserror` in CLI vs lib boundary).
- Keep dependency count small; optional CI: `cargo deny` for advisories and duplicate licenses.
- Release profile: `lto = true`, `strip = true` for smaller distributable binary.

No dependency on Cursor APIs.

## 7. CLI behavior and exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Parse error (malformed plan/spec) |
| 2 | Validation failed (`validate` or `--strict`) |
| 3 | I/O error |

**Strict mode (`--strict`):** fail on unknown spec refs, empty heuristic with no explicit refs, or unresolved **Consumes** with no matching prior **Produces** (warning-only by default).

## 8. Error handling

- **Unreadable paths:** clear message, exit 3
- **Plan with no tasks:** exit 1
- **Duplicate spec IDs:** exit 1 on parse
- **Partial heuristic:** proceed with warnings in manifest and stderr summary table

Orchestrators should pass `--strict` in CI; local iteration uses default warnings.

## 9. Testing strategy

| Area | Tests |
|------|-------|
| Plan parser | Golden files: sample superpowers plan fragments |
| Spec index | IDs from `{#spec:x}` and comments; heading tree |
| Resolver | Explicit-only, heuristic-only, mixed; top-K and truncation |
| Closure | Consumes/Produces across tasks 1→N |
| Bundler | Snapshot of output markdown + manifest JSON |
| CLI | Integration: fixture spec+plan → expected exit code and file count |

Fixtures live under `crates/specometer-core/tests/fixtures/` (no production code).

## 10. Orchestrator contract (any runtime)

1. Run `specometer slice` (or use pre-bound artifacts from CI)
2. For task N, point the implementer at `out/task-NN-<slug>.md` only
3. For task reviewer, optionally provide manifest entry for that task plus the bundle (reviewer does not need full spec if bundle was strict-validated)

Future v2: optional `specometer explain task-3` to print why sections were chosen (debugging heuristics).

## 11. Extension points (post-v1)

- `ResolverPlugin` hook for LLM-assisted section pick with cited line ranges
- Custom ref namespaces beyond `spec:`
- JSON bundle format for non-markdown agents

## 12. Success criteria

- Given a 500+ line spec and 10-task plan, a subagent implementer can complete a task using only its bundle without opening spec/plan
- `validate --strict` passes on plans that use explicit **Spec refs** for every task
- Heuristic-only plans produce useful bundles for ≥80% of tasks in fixture suite (measured manually in fixtures; no LLM)
- Full `cargo test` workspace suite runs in under 5 seconds locally (release-mode resolver tests may be benchmarked separately)
