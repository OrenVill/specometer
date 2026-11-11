#!/usr/bin/env bash
# Install specometer binary and link specometer-dispatch into agent skill dirs.
set -euo pipefail

REPO_URL="${SPECOMETER_REPO:-https://github.com/orenvill/specometer.git}"
PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${PREFIX}/bin"
ALL_AGENTS=0
NONINTERACTIVE=0
AGENTS_CSV=""
SKILLS_ONLY=0
BIN_ONLY=0
CLONE_TMP=""

usage() {
  cat <<EOF
Usage: ./scripts/install.sh [options]

Install specometer CLI, then interactively choose which agents receive
specometer-dispatch (Cursor, Claude Code, Codex, …).

Options:
  --prefix PATH          Install binary to PREFIX/bin (default: ~/.local)
  --agents ID,ID         Non-interactive, e.g. cursor,claude,codex (see skills/.../agents.json)
  --all-agents           Link all supported agents (creates missing project dirs)
  --non-interactive      Link only where agent folders already exist; no prompts
  --skills-only          Skip binary install
  --bin-only             Skip skill links
  --from-git             Clone repo to a temp dir and install from there
  -h, --help             Show help

Quick start:
  git clone --depth 1 ${REPO_URL}
  cd specometer
  ./scripts/install.sh

Ensure PREFIX/bin is on PATH.
EOF
}

FROM_GIT=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      PREFIX="$2"
      BIN_DIR="${PREFIX}/bin"
      shift 2
      ;;
    --all-agents) ALL_AGENTS=1; NONINTERACTIVE=1; shift ;;
    --non-interactive) NONINTERACTIVE=1; shift ;;
    --agents)
      AGENTS_CSV="$2"
      NONINTERACTIVE=1
      shift 2
      ;;
    --skills-only) SKILLS_ONLY=1; shift ;;
    --bin-only) BIN_ONLY=1; shift ;;
    --from-git) FROM_GIT=1; shift ;;
    --create-agent-dirs)
      echo "note: --create-agent-dirs is deprecated; use interactive install or --all-agents" >&2
      ALL_AGENTS=1
      NONINTERACTIVE=1
      shift
      ;;
    --user)
      echo "note: --user is deprecated; choose 'Cursor (global)' in the interactive menu" >&2
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

resolve_root() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local candidate="$(cd "${script_dir}/.." && pwd)"
  if [[ -f "${candidate}/Cargo.toml" ]] && grep -q specometer-core "${candidate}/Cargo.toml" 2>/dev/null; then
    echo "${candidate}"
    return 0
  fi
  return 1
}

cleanup() {
  if [[ -n "${CLONE_TMP}" && -d "${CLONE_TMP}" ]]; then
    rm -rf "${CLONE_TMP}"
  fi
}

ROOT=""
if ROOT="$(resolve_root 2>/dev/null)"; then
  :
elif [[ "${FROM_GIT}" -eq 1 ]]; then
  CLONE_TMP="$(mktemp -d)"
  trap cleanup EXIT
  git clone --depth 1 "${REPO_URL}" "${CLONE_TMP}/specometer"
  ROOT="${CLONE_TMP}/specometer"
else
  echo "error: not inside specometer checkout; use --from-git or clone the repository" >&2
  exit 1
fi

# shellcheck source=lib/install-skills.sh
source "${ROOT}/scripts/lib/install-skills.sh"

install_binary() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found. Install Rust: https://rustup.rs" >&2
    exit 1
  fi
  echo "building specometer..."
  cargo build --release --manifest-path "${ROOT}/Cargo.toml"
  mkdir -p "${BIN_DIR}"
  install -m 755 "${ROOT}/target/release/specometer" "${BIN_DIR}/specometer"
  echo "binary → ${BIN_DIR}/specometer"
}

if [[ "${SKILLS_ONLY}" -eq 0 ]]; then
  install_binary
fi

if [[ "${BIN_ONLY}" -eq 0 ]]; then
  selected=""
  if [[ -n "${AGENTS_CSV}" ]]; then
    selected="${AGENTS_CSV//,/ }"
    specometer_install_skills_selected "${ROOT}" "${selected}" || exit 1
  elif [[ "${NONINTERACTIVE}" -eq 1 ]]; then
    specometer_install_skills_noninteractive "${ROOT}" "${ALL_AGENTS}" || {
      echo "hint: run without --non-interactive to pick agents in a short menu" >&2
      exit 1
    }
  else
    specometer_prompt_agent_selection "${ROOT}"
    specometer_install_skills_selected "${ROOT}" "${SPECOMETER_SELECTED_AGENT_IDS}" || true
  fi
fi

echo "done."
