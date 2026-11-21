#!/usr/bin/env bash
# Assemble a portable Windows build from a dist/windows directory produced by
# build-windows.ps1. Run on Linux for artifact inspection only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist/windows"
OUT="$ROOT/dist/LiveLinguist-0.1.0-windows-x64-portable.zip"
test -d "$DIST" || { echo "missing $DIST (run scripts/build-windows.ps1 on Windows)"; exit 1; }
rm -f "$OUT"
(cd "$DIST" && zip -r "$OUT" .)
echo "built $OUT"
