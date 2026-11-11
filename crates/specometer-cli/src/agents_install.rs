use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use dialoguer::{theme::ColorfulTheme, MultiSelect};

const SKILL_NAME: &str = "specometer-dispatch";

struct AgentDef {
    id: &'static str,
    label: &'static str,
    /// Empty = user-global `~/.cursor/skills`
    parent: &'static str,
    sub: &'static str,
}

const AGENTS: &[AgentDef] = &[
    AgentDef {
        id: "cursor",
        label: "Cursor",
        parent: ".cursor",
        sub: "skills",
    },
    AgentDef {
        id: "claude",
        label: "Claude Code",
        parent: ".claude",
        sub: "skills",
    },
    AgentDef {
        id: "codex",
        label: "Codex",
        parent: ".codex",
        sub: "skills",
    },
    AgentDef {
        id: "opencode",
        label: "OpenCode",
        parent: ".opencode",
        sub: "skills",
    },
    AgentDef {
        id: "continue",
        label: "Continue",
        parent: ".continue",
        sub: "skills",
    },
    AgentDef {
        id: "aider",
        label: "Aider",
        parent: ".aider",
        sub: "skills",
    },
    AgentDef {
        id: "user",
        label: "Cursor (global)",
        parent: "",
        sub: "",
    },
];

fn skill_source(repo: &Path) -> PathBuf {
    repo.join("skills").join(SKILL_NAME)
}

fn agent_present(repo: &Path, agent: &AgentDef) -> bool {
    if agent.parent.is_empty() {
        return dirs::home_dir()
            .map(|h| h.join(".cursor").join("skills").is_dir())
            .unwrap_or(false);
    }
    repo.join(agent.parent).is_dir()
}

fn item_label(repo: &Path, agent: &AgentDef) -> String {
    let where_ = if agent.parent.is_empty() {
        "~/.cursor/skills".to_string()
    } else {
        format!("{}/{}", agent.parent, agent.sub)
    };
    let status = if agent_present(repo, agent) {
        "found"
    } else {
        "new"
    };
    format!("{label:<14} {where_:<22} [{status}]", label = agent.label)
}

#[cfg(unix)]
fn symlink_dir(source: &Path, dest: &Path) -> Result<(), i32> {
    if dest.symlink_metadata().is_ok() {
        if dest.is_dir() && !dest.is_symlink() {
            fs::remove_dir_all(dest).map_err(io_exit)?;
        } else {
            fs::remove_file(dest).map_err(io_exit)?;
        }
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(io_exit)?;
    }
    std::os::unix::fs::symlink(source, dest).map_err(io_exit)?;
    Ok(())
}

#[cfg(not(unix))]
fn symlink_dir(_source: &Path, _dest: &Path) -> Result<(), i32> {
    eprintln!("error: skill install requires a Unix system");
    Err(1)
}

fn io_exit(e: std::io::Error) -> i32 {
    eprintln!("io error: {e}");
    3
}

pub fn link_agents(repo: &Path, ids: &[&str]) -> Result<(), i32> {
    let source = skill_source(repo);
    if !source.join("SKILL.md").is_file() {
        eprintln!("error: missing {}", source.join("SKILL.md").display());
        return Err(1);
    }
    let source = fs::canonicalize(&source).map_err(io_exit)?;

    let mut linked = 0usize;
    for id in ids {
        let Some(agent) = AGENTS.iter().find(|a| a.id == *id) else {
            eprintln!("warning: unknown agent id {id}");
            continue;
        };
        let dest = if agent.parent.is_empty() {
            let home = dirs::home_dir().ok_or_else(|| {
                eprintln!("error: cannot resolve home directory");
                3
            })?;
            home.join(".cursor/skills").join(SKILL_NAME)
        } else {
            repo.join(agent.parent)
                .join(agent.sub)
                .join(SKILL_NAME)
        };
        symlink_dir(&source, &dest)?;
        println!("skill → {}", dest.display());
        linked += 1;
    }

    if linked > 0 {
        append_agents_md(repo);
    } else {
        println!("no agents selected; skipped skill install");
    }
    Ok(())
}

fn append_agents_md(repo: &Path) {
    let agents_md = repo.join("AGENTS.md");
    if !agents_md.is_file() {
        return;
    }
    let Ok(content) = fs::read_to_string(&agents_md) else {
        return;
    };
    if content.contains("specometer-dispatch") {
        return;
    }
    let appendix = "\n## specometer\n\nAfter saving an implementation plan, run `./scripts/slice-project-docs.sh`. Dispatch subagents with a single bundle under `.specometer/bundles/` only. Skill: `skills/specometer-dispatch/SKILL.md`.\n";
    if fs::OpenOptions::new()
        .append(true)
        .open(&agents_md)
        .and_then(|mut f| std::io::Write::write_all(&mut f, appendix.as_bytes()))
        .is_ok()
    {
        println!("appended specometer section to AGENTS.md");
    }
}

pub fn ids_existing_dirs(repo: &Path) -> Vec<&'static str> {
    AGENTS
        .iter()
        .filter(|a| agent_present(repo, a))
        .map(|a| a.id)
        .collect()
}

pub fn all_agent_ids() -> Vec<&'static str> {
    AGENTS.iter().map(|a| a.id).collect()
}

pub fn run_agents_install(
    repo: &Path,
    agents: Option<&str>,
    all_agents: bool,
    non_interactive: bool,
) -> Result<(), i32> {
    let repo = fs::canonicalize(repo).map_err(io_exit)?;

    let selected: Vec<&str> = if let Some(csv) = agents {
        csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else if all_agents {
        all_agent_ids()
    } else if non_interactive || !IsTerminal::is_terminal(&std::io::stdin()) {
        ids_existing_dirs(&repo)
    } else {
        run_tui(&repo)?
    };

    link_agents(&repo, &selected)
}

fn run_tui(repo: &Path) -> Result<Vec<&'static str>, i32> {
    let items: Vec<String> = AGENTS.iter().map(|a| item_label(repo, a)).collect();
    let defaults: Vec<bool> = AGENTS
        .iter()
        .map(|a| agent_present(repo, a))
        .collect();

    let theme = ColorfulTheme::default();
    let picked = MultiSelect::with_theme(&theme)
        .with_prompt("specometer-dispatch — Space toggle, Enter confirm")
        .items(&items)
        .defaults(&defaults)
        .interact()
        .map_err(|e| {
            eprintln!("cancelled: {e}");
            1
        })?;

    Ok(picked.iter().map(|&i| AGENTS[i].id).collect())
}
