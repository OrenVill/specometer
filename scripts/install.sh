#!/usr/bin/env bash
# Install specometer binary and link specometer-dispatch into agent skill dirs.
set -euo pipefail

REPO_URL="${SPECOMETER_REPO:-https://github.com/orenvill/specometer.git}"
PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${PREFIX}/bin"
CREATE_AGENT_DIRS=0
USER_SKILLS=0
SKILLS_ONLY=0
BIN_ONLY=0
CLONE_TMP=""

usage() {
  cat <<EOF
Usage: ./scripts/install.sh [options]

Install specometer CLI and link specometer-dispatch into agent skill directories
when those agents are already set up in the project (.cursor, .claude, .codex, …).

Options:
  --prefix PATH          Install binary to PREFIX/bin (default: ~/.local)
  --create-agent-dirs    Create .cursor / .claude / .codex / … if missing
  --user                 Also link skill to ~/.cursor/skills/
  --skills-only          Skip binary install
  --bin-only             Skip skill links
  -h, --help             Show help

Quick start:
  git clone --depth 1 ${REPO_URL}
  cd specometer
  ./scripts/install.sh --create-agent-dirs

Ensure PREFIX/bin is on PATH, e.g. export PATH="\${HOME}/.local/bin:\$PATH".

Or install from GitHub without cloning first:
  curl -fsSL https://raw.githubusercontent.com/orenvill/specometer/master/scripts/install.sh | bash -s -- --from-git
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
    --create-agent-dirs) CREATE_AGENT_DIRS=1; shift ;;
    --user) USER_SKILLS=1; shift ;;
    --skills-only) SKILLS_ONLY=1; shift ;;
    --bin-only) BIN_ONLY=1; shift ;;
    --from-git) FROM_GIT=1; shift ;;
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
  if ! specometer_install_skills "${ROOT}" "${CREATE_AGENT_DIRS}" "${USER_SKILLS}"; then
    if [[ "${CREATE_AGENT_DIRS}" -eq 0 ]]; then
      echo "hint: re-run with --create-agent-dirs to register the skill for Cursor / Claude / Codex" >&2
    fi
    exit 1
  fi
fi

echo "done."
