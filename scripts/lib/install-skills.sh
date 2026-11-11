#!/usr/bin/env bash
# Non-interactive skill install via specometer CLI (interactive uses agents-install TUI).
set -euo pipefail

specometer_install_skills_selected() {
  local root="${1:?repo root}"
  local agents_csv="${2:-}"
  local specometer_bin="${3:?specometer binary}"

  local args=(agents-install --repo "${root}")
  if [[ -n "${agents_csv}" ]]; then
    args+=(--agents "${agents_csv}")
  else
    args+=(--non-interactive)
  fi
  "${specometer_bin}" "${args[@]}"
}

specometer_install_skills_all() {
  local root="$1"
  local specometer_bin="$2"
  "${specometer_bin}" agents-install --repo "${root}" --all-agents
}
