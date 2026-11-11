# Agent skills (specometer)

| Skill | Purpose |
|-------|---------|
| [specometer-dispatch](specometer-dispatch/SKILL.md) | After **writing-plans**, run `scripts/slice-project-docs.sh`; subagents read `.specometer/bundles/task-*.md` only |

## Install (binary + skills)

```bash
./scripts/install.sh
```

- Installs `specometer` to `~/.local/bin`
- **Interactive menu**: toggle agents with numbers `1`–`6`, `a` = all, `c` = confirm, `q` = skip skills
- Creates `.cursor/skills`, `.claude/skills`, etc. only for agents you checked

Non-interactive: `./scripts/install.sh --agents cursor,claude` or `--all-agents`

No clone:

```bash
curl -fsSL https://raw.githubusercontent.com/orenvill/specometer/master/scripts/install.sh | bash -s -- --from-git
```

In **another project** that uses Cursor/Claude: add specometer as a submodule or vendored `skills/`, then run install from that tree with `--skills-only`, or copy `skills/specometer-dispatch` and run install from a full specometer checkout with `--prefix`.
