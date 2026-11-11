# Specometer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust `specometer` CLI and `specometer-core` library that slice a design spec and superpowers-style plan into per-task context bundles plus `manifest.json`.

**Architecture:** A Cargo workspace with `specometer-core` (parse → index → resolve → bundle) and `specometer-cli` (`clap` commands `slice`, `bind`, `validate`). Markdown is parsed with `pulldown-cmark`; outputs are deterministic files only (no network).

**Tech Stack:** Rust edition 2021, MSRV 1.78+, `pulldown-cmark`, `serde`/`serde_json`, `sha2`, `clap`, `anyhow`/`thiserror`, `insta`.

## Global Constraints

- Rust edition **2021**, MSRV **1.78+**; core crate `#![forbid(unsafe_code)]`.
- Workspace layout: `crates/specometer-core`, `crates/specometer-cli`; CLI binary name **`specometer`**.
- Spec section IDs: `^spec:[a-z0-9][a-z0-9-]*$` via `{#spec:id}` or `<!-- id: spec:... -->` on the line after a heading.
- Plan tasks: `### Task N: Title`; optional `**Spec refs:** spec:foo, spec:bar`.
- Resolution: explicit refs first; heuristic when empty or `--heuristic=always` (default `when-no-explicit`); top-K=3, max heuristic sections=5, max lines per section=200.
- Bundle section order: front matter → Global Constraints → task block → Spec context → Prior task interfaces → dispatch note.
- Exit codes: 0 success, 1 parse, 2 validation, 3 I/O.
- **No** `cursor/` paths, Cursor APIs, or co-authored commits.
- Release profile: `lto = true`, `strip = true` in workspace `Cargo.toml`.
- No network I/O in the CLI.

## Target file map

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Workspace members, shared `release` profile |
| `rust-toolchain.toml` | Pin `1.78` |
| `crates/specometer-core/Cargo.toml` | Core deps |
| `crates/specometer-core/src/lib.rs` | Re-exports |
| `crates/specometer-core/src/error.rs` | `SpecometerError` |
| `crates/specometer-core/src/line_span.rs` | 1-based line ranges in source text |
| `crates/specometer-core/src/spec_index.rs` | `SpecIndex`, sections, IDs |
| `crates/specometer-core/src/plan_index.rs` | `PlanIndex`, tasks, global constraints |
| `crates/specometer-core/src/resolver.rs` | `Resolver`, heuristic scoring |
| `crates/specometer-core/src/closure.rs` | Prior-task interface closure |
| `crates/specometer-core/src/bundle.rs` | Markdown bundle + manifest types |
| `crates/specometer-core/src/pipeline.rs` | `run_slice` orchestration |
| `crates/specometer-cli/Cargo.toml` | Depends on core + clap |
| `crates/specometer-cli/src/main.rs` | Subcommands, exit codes |
| `crates/specometer-core/tests/fixtures/*` | Golden spec/plan fragments |
| `README.md` | Install, usage, orchestrator contract |

---

### Task 1: Cargo workspace scaffold

**Spec refs:** spec:architecture, spec:tech-stack

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `crates/specometer-core/Cargo.toml`, `crates/specometer-core/src/lib.rs`, `crates/specometer-core/src/error.rs`, `crates/specometer-cli/Cargo.toml`, `crates/specometer-cli/src/main.rs`
- Test: `crates/specometer-core/tests/smoke.rs`

**Interfaces:**
- Consumes: (none)
- Produces: `specometer_core::VERSION: &'static str`, `SpecometerError` enum, CLI stub exits 0 on `--version`

- [ ] **Step 1: Write the failing test**

Create `crates/specometer-core/tests/smoke.rs`:

```rust
use specometer_core::VERSION;

#[test]
fn version_is_non_empty() {
    assert!(!VERSION.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core smoke::version_is_non_empty`
Expected: FAIL (crate missing or `VERSION` undefined)

- [ ] **Step 3: Write minimal implementation**

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/specometer-core", "crates/specometer-cli"]

[profile.release]
lto = true
strip = true
```

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.78"
```

`crates/specometer-core/Cargo.toml`:

```toml
[package]
name = "specometer-core"
version = "0.1.0"
edition = "2021"

[dependencies]
pulldown-cmark = "0.11"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
thiserror = "2"

[dev-dependencies]
insta = { version = "1", features = ["json"] }
```

`crates/specometer-core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod error;
pub mod line_span;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::SpecometerError;
```

`crates/specometer-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecometerError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

`crates/specometer-cli/Cargo.toml`:

```toml
[package]
name = "specometer-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "specometer"
path = "src/main.rs"

[dependencies]
specometer-core = { path = "../specometer-core" }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

`crates/specometer-cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "specometer", version = specometer_core::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version (stub until slice is implemented)
    Stub,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Stub => println!("{}", specometer_core::VERSION),
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml crates/
git commit -m "chore: add specometer Cargo workspace scaffold"
```

---

### Task 2: Spec index and section IDs

**Spec refs:** spec:conventions, spec:resolution

**Files:**
- Create: `crates/specometer-core/src/spec_index.rs`
- Modify: `crates/specometer-core/src/lib.rs`
- Test: `crates/specometer-core/tests/spec_index.rs`
- Create: `crates/specometer-core/tests/fixtures/spec/minimal.md`

**Interfaces:**
- Consumes: `LineSpan`, `SpecometerError`
- Produces:
  - `pub struct SpecSection { pub id: Option<String>, pub heading: String, pub level: u8, pub lines: LineSpan, pub body_lines: LineSpan }`
  - `pub struct SpecIndex { pub sections: Vec<SpecSection> }`
  - `pub fn SpecIndex::from_markdown(src: &str) -> Result<SpecIndex, SpecometerError>`

- [ ] **Step 1: Write the failing test**

Create `crates/specometer-core/tests/fixtures/spec/minimal.md`:

```markdown
# Product

## Authentication {#spec:auth}

Users sign in with email.

## Errors
<!-- id: spec:errors -->

Use problem+json.
```

Create `crates/specometer-core/tests/spec_index.rs`:

```rust
use specometer_core::spec_index::SpecIndex;

#[test]
fn indexes_pandoc_and_comment_ids() {
    let src = include_str!("fixtures/spec/minimal.md");
    let index = SpecIndex::from_markdown(src).unwrap();
  let ids: Vec<_> = index
      .sections
      .iter()
      .filter_map(|s| s.id.as_deref())
      .collect();
  assert_eq!(ids, vec!["spec:auth", "spec:errors"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core spec_index::indexes_pandoc_and_comment_ids`
Expected: FAIL (`spec_index` module missing)

- [ ] **Step 3: Write minimal implementation**

`crates/specometer-core/src/line_span.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
}

impl LineSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        let mut line = 1usize;
        let mut start_byte = 0usize;
        let mut end_byte = src.len();
        for (i, b) in src.bytes().enumerate() {
            if line == self.start && start_byte == 0 {
                start_byte = i;
            }
            if line == self.end + 1 {
                end_byte = i;
                break;
            }
            if b == b'\n' {
                line += 1;
            }
        }
        &src[start_byte..end_byte]
    }
}
```

`crates/specometer-core/src/spec_index.rs` — implement `from_markdown`:

- Split spec into lines; scan for `^#{1,6}\s+` headings.
- Parse `{#spec:...}` on same line with regex `spec:[a-z0-9][a-z0-9-]*`.
- Else if next line matches `<!-- id: spec:... -->`, attach ID.
- Reject duplicate IDs with `SpecometerError::Parse`.
- Section body runs until next heading of level ≤ current.

Export in `lib.rs`: `pub mod spec_index;`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core spec_index`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-core/
git commit -m "feat(core): parse spec heading index with section IDs"
```

---

### Task 3: Plan index (tasks, global constraints, spec refs)

**Spec refs:** spec:conventions

**Files:**
- Create: `crates/specometer-core/src/plan_index.rs`
- Modify: `crates/specometer-core/src/lib.rs`
- Test: `crates/specometer-core/tests/plan_index.rs`
- Create: `crates/specometer-core/tests/fixtures/plan/minimal.md`

**Interfaces:**
- Consumes: `LineSpan`, `SpecometerError`
- Produces:
  - `pub struct PlanTask { pub number: u32, pub title: String, pub slug: String, pub spec_refs: Vec<String>, pub raw_block: String, pub lines: LineSpan, pub interfaces_raw: Option<String> }`
  - `pub struct PlanIndex { pub global_constraints: String, pub tasks: Vec<PlanTask> }`
  - `pub fn PlanIndex::from_markdown(src: &str) -> Result<PlanIndex, SpecometerError>`
  - `pub fn slugify_task_title(number: u32, title: &str) -> String` → `task-03-login-api` form (NN zero-padded, kebab, max 48 char slug)

- [ ] **Step 1: Write the failing test**

`crates/specometer-core/tests/fixtures/plan/minimal.md`:

```markdown
# Feature Plan

## Global Constraints

- Rust only

### Task 1: Scaffold

**Spec refs:** spec:auth

- [ ] step

### Task 2: API

**Files:**
- Create: `src/api.rs`
```

`crates/specometer-core/tests/plan_index.rs`:

```rust
use specometer_core::plan_index::PlanIndex;

#[test]
fn parses_global_constraints_and_tasks() {
    let src = include_str!("fixtures/plan/minimal.md");
    let plan = PlanIndex::from_markdown(src).unwrap();
    assert!(plan.global_constraints.contains("Rust only"));
    assert_eq!(plan.tasks.len(), 2);
    assert_eq!(plan.tasks[0].spec_refs, vec!["spec:auth".to_string()]);
    assert_eq!(plan.tasks[0].slug, "task-01-scaffold");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core plan_index::parses_global_constraints_and_tasks`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

- Locate `## Global Constraints` (spec header) through next `---` or first `### Task`.
- Split on `^### Task (\d+): (.+)$` (multiline).
- Within each block, parse `**Spec refs:**` line into comma-split IDs.
- Capture full `raw_block` text and `interfaces_raw` from `**Interfaces:**` subsection through next `**Foo:**` or end.
- `slugify_task_title`: lowercase, non-alphanumeric → `-`, trim, truncate.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core plan_index`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-core/
git commit -m "feat(core): parse plan tasks and global constraints"
```

---

### Task 4: Resolver (explicit + heuristic)

**Spec refs:** spec:resolution

**Files:**
- Create: `crates/specometer-core/src/resolver.rs`
- Modify: `crates/specometer-core/src/lib.rs`
- Test: `crates/specometer-core/tests/resolver.rs`

**Interfaces:**
- Consumes: `SpecIndex`, `PlanTask`, `PlanIndex::global_constraints`
- Produces:
  - `pub enum SectionPickMode { Explicit, Heuristic }`
  - `pub struct PickedSection { pub section_index: usize, pub mode: SectionPickMode, pub score: Option<f32>, pub warning: Option<String> }`
  - `pub struct ResolverConfig { pub heuristic: HeuristicMode, pub top_k: usize, pub min_score: f32, pub max_sections: usize, pub max_section_lines: usize }`
  - `pub enum HeuristicMode { WhenNoExplicit, Always, Never }`
  - `pub fn resolve_sections(spec: &SpecIndex, plan: &PlanIndex, task: &PlanTask, cfg: &ResolverConfig) -> Result<Vec<PickedSection>, SpecometerError>`

- [ ] **Step 1: Write the failing test**

```rust
use specometer_core::plan_index::PlanIndex;
use specometer_core::resolver::{HeuristicMode, ResolverConfig, resolve_sections};
use specometer_core::spec_index::SpecIndex;

#[test]
fn explicit_refs_win_over_heuristic() {
    let spec = SpecIndex::from_markdown(include_str!("fixtures/spec/minimal.md")).unwrap();
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/minimal.md")).unwrap();
    let cfg = ResolverConfig::default();
    let picks = resolve_sections(&spec, &plan, &plan.tasks[0], &cfg).unwrap();
    assert!(picks.iter().all(|p| matches!(p.mode, specometer_core::resolver::SectionPickMode::Explicit)));
    assert_eq!(picks.len(), 1);
}

#[test]
fn heuristic_fills_when_no_refs() {
    let spec = SpecIndex::from_markdown(include_str!("fixtures/spec/minimal.md")).unwrap();
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/minimal.md")).unwrap();
    let cfg = ResolverConfig::default();
    let picks = resolve_sections(&spec, &plan, &plan.tasks[1], &cfg).unwrap();
    assert!(!picks.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core resolver`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

- Explicit: map each `task.spec_refs` to section by ID; error on unknown ID only when caller sets strict (validation layer); in resolver return `SpecometerError::Validation` for unknown if `cfg.strict_unknown_refs` (add bool, default false).
- Heuristic: tokenize title + file paths from **Files** lines + interface lines; score heading overlap + first paragraph; boost path/heading slug match; merge global-constraint phrase hits.
- Sort by score desc, take `top_k`, cap `max_sections`, attach `score` on heuristic picks.
- Truncate section line span to `max_section_lines`, set `warning` when truncated.

Default `ResolverConfig`:

```rust
impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            heuristic: HeuristicMode::WhenNoExplicit,
            top_k: 3,
            min_score: 0.15,
            max_sections: 5,
            max_section_lines: 200,
            strict_unknown_refs: false,
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core resolver`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-core/
git commit -m "feat(core): explicit and heuristic spec section resolver"
```

---

### Task 5: Prior-task interface closure

**Spec refs:** spec:resolution, spec:orchestrator

**Files:**
- Create: `crates/specometer-core/src/closure.rs`
- Modify: `crates/specometer-core/src/lib.rs`
- Test: `crates/specometer-core/tests/closure.rs`
- Create: `crates/specometer-core/tests/fixtures/plan/interfaces.md`

**Interfaces:**
- Consumes: `PlanIndex`, `PlanTask`
- Produces: `pub fn prior_interface_blocks(plan: &PlanIndex, task: &PlanTask) -> Vec<String>` (markdown snippets)

- [ ] **Step 1: Write the failing test**

`fixtures/plan/interfaces.md`:

```markdown
## Global Constraints

- none

### Task 1: Types

**Interfaces:**
- Produces: `pub struct UserId(pub u64);`

### Task 2: API

**Interfaces:**
- Consumes: `UserId`

- Produces: `pub fn get_user(id: UserId) -> User;`
```

```rust
use specometer_core::closure::prior_interface_blocks;
use specometer_core::plan_index::PlanIndex;

#[test]
fn includes_earlier_produces_matching_consumes() {
    let plan = PlanIndex::from_markdown(include_str!("fixtures/plan/interfaces.md")).unwrap();
    let blocks = prior_interface_blocks(&plan, &plan.tasks[1]);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("UserId"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core closure`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

- Parse bullet tokens after `- Consumes:` / `- Produces:` (backtick-delimited symbols).
- For task index `i`, scan tasks `0..i`; if any produce token matches any consume token (substring match on symbol text), push that earlier task’s `interfaces_raw` or full **Interfaces** block.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core closure`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-core/
git commit -m "feat(core): prior-task interface closure"
```

---

### Task 6: Bundler, manifest, and pipeline

**Spec refs:** spec:bundle-schema

**Files:**
- Create: `crates/specometer-core/src/bundle.rs`, `crates/specometer-core/src/pipeline.rs`
- Modify: `crates/specometer-core/src/lib.rs`
- Test: `crates/specometer-core/tests/bundle_snap.rs`

**Interfaces:**
- Consumes: `resolve_sections`, `prior_interface_blocks`, `SpecIndex`, `PlanIndex`, `sha2`
- Produces:
  - `pub struct Manifest { pub spec: FileRef, pub plan: FileRef, pub tasks: Vec<ManifestTask> }`
  - `pub struct SliceOutput { pub bundles: Vec<(String, String)>, pub manifest: Manifest }`
  - `pub fn run_slice(spec_src: &str, plan_src: &str, spec_path: &str, plan_path: &str, cfg: &ResolverConfig) -> Result<SliceOutput, SpecometerError>`

- [ ] **Step 1: Write the failing test**

```rust
use insta::assert_json_snapshot;
use specometer_core::pipeline::run_slice;
use specometer_core::resolver::ResolverConfig;

#[test]
fn snapshot_manifest_for_minimal_fixtures() {
    let out = run_slice(
        include_str!("fixtures/spec/minimal.md"),
        include_str!("fixtures/plan/minimal.md"),
        "minimal.md",
        "minimal.md",
        &ResolverConfig::default(),
    )
    .unwrap();
    assert_eq!(out.bundles.len(), 2);
    assert_json_snapshot!(out.manifest);
}
```

Enable `insta` in dev-dependencies (already in Task 1).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core bundle_snap`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

`bundle.rs`:

- Build each bundle markdown with YAML front matter (`serde` not required for hand-written minimal YAML or use `serde_yaml` if preferred — keep deps minimal: format manually with `format!`).
- Insert spec excerpts via `LineSpan::slice` with `<!-- from: spec:auth lines 4-8 -->` comments.
- Append `## Prior task interfaces` when non-empty.
- End with static dispatch note paragraph from spec.

`pipeline.rs`:

- Compute SHA256 of spec/plan bytes for manifest.
- Loop tasks, resolve, closure, bundle; collect warnings (empty heuristic, unknown refs).

Run `insta test` once to accept snapshot:

```bash
INSTA_UPDATE=always cargo test -p specometer-core bundle_snap
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-core bundle_snap`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-core/ crates/specometer-core/tests/bundle_snap.rs.snap
git commit -m "feat(core): bundle markdown and manifest pipeline"
```

---

### Task 7: CLI `slice` and `bind`

**Spec refs:** spec:cli

**Files:**
- Modify: `crates/specometer-cli/src/main.rs`
- Test: `crates/specometer-cli/tests/cli_slice.rs`

**Interfaces:**
- Consumes: `specometer_core::pipeline::run_slice`, `ResolverConfig`
- Produces: CLI writing `out/task-*.md` and `out/manifest.json`; `bind` also writes `.specometer/manifest.json`

- [ ] **Step 1: Write the failing test**

`crates/specometer-cli/Cargo.toml` add:

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

`crates/specometer-cli/tests/cli_slice.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn slice_writes_bundles_and_manifest() {
    let spec = concat!(env!("CARGO_MANIFEST_DIR"), "/../specometer-core/tests/fixtures/spec/minimal.md");
    let plan = concat!(env!("CARGO_MANIFEST_DIR"), "/../specometer-core/tests/fixtures/plan/minimal.md");
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("specometer")
        .unwrap()
        .args([
            "slice",
            "--spec",
            spec,
            "--plan",
            plan,
            "--out",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let manifest = temp.path().join("manifest.json");
    assert!(manifest.exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-cli slice_writes_bundles_and_manifest`
Expected: FAIL (unknown subcommand)

- [ ] **Step 3: Write minimal implementation**

Replace `main.rs` with `clap` subcommands:

```rust
#[derive(Subcommand)]
enum Commands {
    Slice {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        plan: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "when-no-explicit")]
        heuristic: String,
        #[arg(long)]
        strict: bool,
    },
    Bind { /* same flags as Slice */ },
}
```

- Read files with `std::fs::read_to_string` (map IO → exit 3 via `process::exit`).
- Call `run_slice`, write each bundle and `manifest.json`.
- `bind`: create `.specometer/` in current dir and copy manifest (per spec).

Map errors: `SpecometerError::Parse` → exit 1, `Validation` → exit 2, `Io` → exit 3.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-cli`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/specometer-cli/
git commit -m "feat(cli): add slice and bind commands"
```

---

### Task 8: CLI `validate` and strict mode

**Spec refs:** spec:cli, spec:errors

**Files:**
- Modify: `crates/specometer-cli/src/main.rs`, `crates/specometer-core/src/resolver.rs`, `crates/specometer-core/src/pipeline.rs`
- Test: `crates/specometer-cli/tests/cli_validate.rs`

**Interfaces:**
- Consumes: `run_slice` with `strict_unknown_refs: true`, validation pass collecting warnings
- Produces: `validate` subcommand exit 2 on strict failures; stderr summary table of warnings

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn validate_strict_fails_on_unknown_spec_ref() {
    let temp = tempfile::tempdir().unwrap();
    let bad_plan = temp.path().join("bad.md");
    std::fs::write(
        &bad_plan,
        "## Global Constraints\n\nx\n\n### Task 1: X\n\n**Spec refs:** spec:missing\n",
    )
    .unwrap();
    let spec = concat!(env!("CARGO_MANIFEST_DIR"), "/../specometer-core/tests/fixtures/spec/minimal.md");
    Command::cargo_bin("specometer")
        .unwrap()
        .args(["validate", "--spec", spec, "--plan", bad_plan.to_str().unwrap(), "--strict"])
        .assert()
        .code(2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-cli validate_strict_fails_on_unknown_spec_ref`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

- Add `Validate` subcommand (no `--out`).
- `pipeline::validate(spec, plan, cfg) -> Result<Vec<Warning>, SpecometerError>` reusing parsers + resolver in dry-run.
- `--strict`: unknown refs, empty heuristic with no explicit refs, unresolved consumes → aggregate `Validation` error.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p specometer-cli validate`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/
git commit -m "feat(cli): validate command and strict mode"
```

---

### Task 9: README and workspace verification

**Spec refs:** spec:orchestrator, spec:success, spec:testing

**Files:**
- Create: `README.md`
- Modify: `.devbox/README.md` (optional one-line pointer)

**Interfaces:**
- Consumes: finished CLI
- Produces: documented install (`cargo install --path crates/specometer-cli`), examples, orchestrator contract

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn readme_exists() {
    let readme = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    assert!(readme.exists(), "README.md required at repo root");
}
```

Place in `crates/specometer-core/tests/readme.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p specometer-core readme`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Root `README.md` with:

- `cargo build --release` and `target/release/specometer slice --spec ... --plan ... --out bundles/`
- Spec/plan conventions (link to design spec)
- Exit codes and `--strict`
- Subagent workflow: pass only `bundles/task-NN-*.md`

- [ ] **Step 4: Run full workspace verification**

Run: `cargo test --workspace`
Expected: all PASS, total time under 15s on dev machine

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS (fix any lints inline)

- [ ] **Step 5: Commit**

```bash
git add README.md crates/specometer-core/tests/readme.rs .devbox/README.md
git commit -m "docs: add specometer usage README"
```

---

## Spec coverage checklist

| Spec section | Task |
|--------------|------|
| Spec/plan parsing | 2, 3 |
| Explicit + heuristic resolution | 4 |
| Cross-task closure | 5 |
| Bundle + manifest schema | 6 |
| slice / bind / validate | 7, 8 |
| Exit codes, strict, security (no network) | 7, 8 |
| Fixtures + tests | 2–9 |
| Rust workspace, forbid unsafe | 1 |
| Release LTO/strip | 1 |

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-11-11-specometer.md`.

**1. Subagent-Driven (recommended)** — fresh subagent per task, review between tasks.

**2. Inline Execution** — implement tasks in this session with checkpoints.

Which approach do you want?
