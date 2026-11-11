# devbox (specometer)

```bash
cd ~/code/specometer
devbox up
```

Starts a My Machines worker on this repo. No dev server until you add `package.json` and uncomment `dev` / `ports` in `project.yaml`.

When you add an app, extend `project.yaml` with `dev.command`, `dev.readyPort`, and a `ports` entry (see `mcp-detective/.devbox/project.yaml`).

For Vite, use `server: { host: true, port: 5173 }` so Docker port publish works (same as mcp-detective).

Optional secrets: copy `secrets.env.example` → `secrets.env` when you add `env.secretsFile` to `project.yaml` (see `~/code/devbox/docs/secrets.md`).
