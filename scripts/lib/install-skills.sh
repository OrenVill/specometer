#!/usr/bin/env bash
# Link skills/specometer-dispatch into agent skill folders (interactive or explicit).
set -euo pipefail

SPECOMETER_SKILL_NAME="specometer-dispatch"

# id | label | project_parent_dir | skills_subdir (user-scope uses empty parent)
read -r -d '' _AGENT_TABLE <<'EOF' || true
cursor|Cursor|.cursor|skills
claude|Claude Code|.claude|skills
codex|Codex|.codex|skills
opencode|OpenCode|.opencode|skills
continue|Continue|.continue|skills
aider|Aider|.aider|skills
user|Cursor (global ~/.cursor/skills)||
EOF

AGENT_IDS=()
AGENT_LABELS=()
AGENT_PARENTS=()
AGENT_SUBS=()

_load_agents() {
  AGENT_IDS=()
  AGENT_LABELS=()
  AGENT_PARENTS=()
  AGENT_SUBS=()
  local line id label parent sub
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    IFS='|' read -r id label parent sub <<<"${line}"
    AGENT_IDS+=("${id}")
    AGENT_LABELS+=("${label}")
    AGENT_PARENTS+=("${parent}")
    AGENT_SUBS+=("${sub}")
  done <<<"$(echo "${_AGENT_TABLE}" | grep -v '^$')"
}

_agent_index() {
  local id="$1"
  local i
  for i in "${!AGENT_IDS[@]}"; do
    if [[ "${AGENT_IDS[$i]}" == "${id}" ]]; then
      echo "${i}"
      return 0
    fi
  done
  return 1
}

_agent_dir_present() {
  local root="$1"
  local idx="$2"
  local parent="${AGENT_PARENTS[$idx]}"
  if [[ -z "${parent}" ]]; then
    [[ -d "${HOME}/.cursor/skills" ]] && return 0
    return 1
  fi
  [[ -d "${root}/${parent}" ]]
}

# Interactive checklist. Sets global SPECOMETER_SELECTED_AGENT_IDS (space-separated).
specometer_prompt_agent_selection() {
  local root="$1"
  _load_agents

  local -a checked=()
  local i
  for i in "${!AGENT_IDS[@]}"; do
    if _agent_dir_present "${root}" "${i}"; then
      checked[$i]=1
    else
      checked[$i]=0
    fi
  done

  if [[ ! -t 0 ]]; then
    SPECOMETER_SELECTED_AGENT_IDS=""
    for i in "${!AGENT_IDS[@]}"; do
      if [[ "${checked[$i]}" -eq 1 ]]; then
        SPECOMETER_SELECTED_AGENT_IDS+="${AGENT_IDS[$i]} "
      fi
    done
    return 0
  fi

  echo ""
  echo "Specometer — add skill to which agents?"
  echo "  [number] toggle   [c] confirm   [q] skip skills   [a] select all"
  echo ""

  while true; do
    for i in "${!AGENT_IDS[@]}"; do
      local mark=" "
      [[ "${checked[$i]}" -eq 1 ]] && mark="x"
      local where
      if [[ -z "${AGENT_PARENTS[$i]}" ]]; then
        where="${HOME}/.cursor/skills"
      else
        where="${AGENT_PARENTS[$i]}/${AGENT_SUBS[$i]}"
      fi
      local status="will create"
      if _agent_dir_present "${root}" "${i}"; then
        status="found"
      elif [[ -z "${AGENT_PARENTS[$i]}" ]]; then
        status="will create"
      fi
      printf "  [%s] %-2s %-22s %-28s (%s)\n" "${mark}" "$((i + 1))" "${AGENT_LABELS[$i]}" "${where}" "${status}"
    done
    echo ""
    read -r -p "Toggle (1-${#AGENT_IDS[@]}), a, c, or q: " choice
    choice="$(echo "${choice}" | tr '[:upper:]' '[:lower:]')"

    case "${choice}" in
      q)
        SPECOMETER_SELECTED_AGENT_IDS=""
        return 0
        ;;
      c | "")
        break
        ;;
      a)
        for i in "${!AGENT_IDS[@]}"; do checked[$i]=1; done
        ;;
      *)
        local n
        for n in ${choice}; do
          if [[ "${n}" =~ ^[0-9]+$ ]] && ((n >= 1 && n <= ${#AGENT_IDS[@]})); then
            local idx=$((n - 1))
            checked[$idx]=$((1 - checked[idx]))
          fi
        done
        ;;
    esac
    echo ""
  done

  SPECOMETER_SELECTED_AGENT_IDS=""
  for i in "${!AGENT_IDS[@]}"; do
    if [[ "${checked[$i]}" -eq 1 ]]; then
      SPECOMETER_SELECTED_AGENT_IDS+="${AGENT_IDS[$i]} "
    fi
  done
}

_link_one_agent() {
  local root="$1"
  local idx="$2"
  local source="$3"

  if [[ -z "${AGENT_PARENTS[$idx]}" ]]; then
    mkdir -p "${HOME}/.cursor/skills"
    local udest="${HOME}/.cursor/skills/${SPECOMETER_SKILL_NAME}"
    rm -rf "${udest}"
    ln -sfn "${source}" "${udest}"
    echo "skill → ${udest}"
    return 0
  fi

  local parent="${AGENT_PARENTS[$idx]}"
  local sub="${AGENT_SUBS[$idx]}"
  mkdir -p "${root}/${parent}"
  local skills_dir="${root}/${parent}/${sub}"
  mkdir -p "${skills_dir}"
  local dest="${skills_dir}/${SPECOMETER_SKILL_NAME}"
  rm -rf "${dest}"
  ln -sfn "${source}" "${dest}"
  echo "skill → ${dest}"
}

# Args: root, space-separated agent ids (or empty = none)
specometer_install_skills_selected() {
  local root="${1:?repo root}"
  local selected_ids="${2:-}"
  local source="${root}/skills/${SPECOMETER_SKILL_NAME}"

  if [[ ! -f "${source}/SKILL.md" ]]; then
    echo "error: missing ${source}/SKILL.md" >&2
    return 1
  fi

  _load_agents

  if [[ -z "${selected_ids// }" ]]; then
    echo "no agents selected; skipped skill install"
    return 0
  fi

  local linked=0
  local id
  for id in ${selected_ids}; do
    local idx
    idx="$(_agent_index "${id}")" || {
      echo "warning: unknown agent id ${id}" >&2
      continue
    }
    _link_one_agent "${root}" "${idx}" "${source}"
    linked=$((linked + 1))
  done

  if [[ -f "${root}/AGENTS.md" ]] && [[ "${linked}" -gt 0 ]]; then
    if ! grep -q "specometer-dispatch" "${root}/AGENTS.md" 2>/dev/null; then
      cat >>"${root}/AGENTS.md" <<EOF

## specometer

After saving an implementation plan, run \`./scripts/slice-project-docs.sh\`. Dispatch subagents with a single bundle under \`.specometer/bundles/\` only. Skill: \`skills/specometer-dispatch/SKILL.md\`.
EOF
      echo "appended specometer section to AGENTS.md"
    fi
  fi

  if [[ "${linked}" -eq 0 ]]; then
    return 1
  fi
  return 0
}

# Non-interactive: --all or legacy --create-agent-dirs
specometer_install_skills_noninteractive() {
  local root="$1"
  local all="${2:-0}"
  _load_agents
  local ids=""
  local i
  for i in "${!AGENT_IDS[@]}"; do
    if [[ "${all}" -eq 1 ]] || _agent_dir_present "${root}" "${i}"; then
      ids+="${AGENT_IDS[$i]} "
    fi
  done
  specometer_install_skills_selected "${root}" "${ids}"
}
