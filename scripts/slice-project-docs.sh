#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SPEC="${SPEC:-docs/superpowers/specs/2026-11-11-specometer-design.md}"
PLAN="${PLAN:-docs/superpowers/plans/2026-11-11-specometer.md}"
OUT="${OUT:-.specometer/bundles}"

cargo build --quiet --release
mkdir -p "$OUT"
./target/release/specometer validate --spec "$SPEC" --plan "$PLAN" --strict
./target/release/specometer slice --spec "$SPEC" --plan "$PLAN" --out "$OUT"

echo "Bundles written to $OUT/"
ls -1 "$OUT"/*.md 2>/dev/null || true
