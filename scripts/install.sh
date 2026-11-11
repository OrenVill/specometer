#!/usr/bin/env bash
# Install specometer binary and link specometer-dispatch (Rust multi-select TUI).
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

Install specometer, then open a small TUI to pick agents (Cursor, Claude, Codex, …).

Options:
  --prefix PATH          Install binary to PREFIX/bin (default: ~/.local)
  --agents ID,ID         Non-interactive, e.g. cursor,claude
  --all-agents           Install skill for every supported agent
  --non-interactive      Only agents whose folders already exist
  --skills-only          Skip binary install
  --bin-only             Skip skill links
  --from-git             Clone to temp dir and install
  -h, --help

  git clone --depth 1 ${REPO_URL} && cd specometer && ./scripts/install.sh
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
    --create-agent-dirs | --user) shift ;;
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
  echo "error: not inside specometer checkout; use --from-git or clone" >&2
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

SPECOMETER_BIN="${ROOT}/target/release/specometer"

if [[ "${SKILLS_ONLY}" -eq 0 ]]; then
  install_binary
else
  cargo build --release --manifest-path "${ROOT}/Cargo.toml" -q
fi

if [[ "${BIN_ONLY}" -eq 0 ]]; then
  if [[ -n "${AGENTS_CSV}" ]]; then
    specometer_install_skills_selected "${ROOT}" "${AGENTS_CSV}" "${SPECOMETER_BIN}"
  elif [[ "${ALL_AGENTS}" -eq 1 ]]; then
    specometer_install_skills_all "${ROOT}" "${SPECOMETER_BIN}"
  elif [[ "${NONINTERACTIVE}" -eq 1 ]]; then
    specometer_install_skills_selected "${ROOT}" "" "${SPECOMETER_BIN}"
  else
  echo ""
  "${SPECOMETER_BIN}" agents-install --repo "${ROOT}"
  fi
fi

echo "done."
