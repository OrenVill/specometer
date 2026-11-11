# Agent skills (specometer)

| Skill | Purpose |
|-------|---------|
| [specometer-dispatch](specometer-dispatch/SKILL.md) | After **writing-plans**, run `scripts/slice-project-docs.sh`; tell subagents to use `.specometer/bundles/task-*.md` only |

Install:

```bash
cp -r skills/specometer-dispatch ~/.cursor/skills/specometer-dispatch
```

Read `specometer-dispatch/SKILL.md` for paths, env vars, and orchestrator rules.
