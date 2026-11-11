#!/usr/bin/env bash
# Link skills/specometer-dispatch into agent skill folders when those agents are present.
# shellcheck disable=SC2034
set -euo pipefail

SPECOMETER_SKILL_NAME="specometer-dispatch"

# parent_dir:relative_skills_path (under repo root)
AGENT_SKILL_TARGETS=(
  ".cursor:skills"
  ".claude:skills"
  ".codex:skills"
  ".opencode:skills"
  ".continue:skills"
  ".aider:skills"
)

specometer_install_skills() {
  local root="${1:?repo root}"
  local create_missing="${2:-0}"
  local user_scope="${3:-0}"
  local source="${root}/skills/${SPECOMETER_SKILL_NAME}"

  if [[ ! -f "${source}/SKILL.md" ]]; then
    echo "error: missing ${source}/SKILL.md" >&2
    return 1
  fi

  local linked=0
  for entry in "${AGENT_SKILL_TARGETS[@]}"; do
    local parent="${entry%%:*}"
    local sub="${entry#*:}"
    local parent_path="${root}/${parent}"

    if [[ ! -d "${parent_path}" ]]; then
      if [[ "${create_missing}" -eq 1 ]]; then
        mkdir -p "${parent_path}"
      else
        continue
      fi
    fi

    local skills_dir="${parent_path}/${sub}"
    mkdir -p "${skills_dir}"
    local dest="${skills_dir}/${SPECOMETER_SKILL_NAME}"
    rm -rf "${dest}"
    ln -sfn "${source}" "${dest}"
    echo "skill → ${dest}"
    linked=$((linked + 1))
  done

  if [[ "${user_scope}" -eq 1 ]] && [[ -d "${HOME}/.cursor/skills" || "${create_missing}" -eq 1 ]]; then
    mkdir -p "${HOME}/.cursor/skills"
    local udest="${HOME}/.cursor/skills/${SPECOMETER_SKILL_NAME}"
    rm -rf "${udest}"
    ln -sfn "${source}" "${udest}"
    echo "skill → ${udest} (user)"
    linked=$((linked + 1))
  fi

  if [[ -f "${root}/AGENTS.md" ]]; then
    if ! grep -q "specometer-dispatch" "${root}/AGENTS.md" 2>/dev/null; then
      cat >>"${root}/AGENTS.md" <<EOF

## specometer

After saving an implementation plan, run \`./scripts/slice-project-docs.sh\`. Dispatch subagents with a single bundle under \`.specometer/bundles/\` only. Skill: \`skills/specometer-dispatch/SKILL.md\`.
EOF
      echo "appended specometer section to AGENTS.md"
    fi
  fi

  if [[ "${linked}" -eq 0 ]]; then
    echo "no agent folders found (expected one of: .cursor, .claude, .codex, .opencode, .continue, .aider)"
    echo "tip: run with --create-agent-dirs or create e.g. .cursor in your project"
    return 1
  fi

  return 0
}
