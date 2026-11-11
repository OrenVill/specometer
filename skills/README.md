# Agent skills (specometer)

| Skill | Purpose |
|-------|---------|
| [specometer-dispatch](specometer-dispatch/SKILL.md) | After **writing-plans**, run `scripts/slice-project-docs.sh`; subagents read `.specometer/bundles/task-*.md` only |

## Install (binary + skills)

```bash
./scripts/install.sh --create-agent-dirs
```

- Installs `specometer` to `~/.local/bin`
- Symlinks `specometer-dispatch` into `.cursor`, `.claude`, `.codex`, `.opencode`, `.continue`, or `.aider` **when that folder exists** (or creates them with `--create-agent-dirs`)

No clone:

```bash
curl -fsSL https://raw.githubusercontent.com/orenvill/specometer/master/scripts/install.sh | bash -s -- --from-git --create-agent-dirs
```

In **another project** that uses Cursor/Claude: add specometer as a submodule or vendored `skills/`, then run install from that tree with `--skills-only`, or copy `skills/specometer-dispatch` and run install from a full specometer checkout with `--prefix`.
