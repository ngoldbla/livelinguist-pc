#!/usr/bin/env bash
# Full local verification for the Linux dev environment.
# Windows hardware acceptance is separate (see docs/plans/2026-11-23-windows-v1-plan.md).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.bun/bin:$PATH"

echo "==> rust format"
cargo fmt --all -- --check
echo "==> rust clippy"
cargo clippy --workspace --all-targets -- -D warnings
echo "==> rust tests (incl. X11 policy when DISPLAY is available)"
cargo test --workspace
echo "==> eval: mock / wav / simplify-only"
rm -rf target/verify-eval
cargo run -q -p ll-eval -- --scenario mock --language fr-FR --out target/verify-eval/mock
cargo run -q -p ll-eval -- --scenario mock --language fr-FR --wav tests/fixtures/sample.wav --out target/verify-eval/wav
cargo run -q -p ll-eval -- --scenario simplify-only --language fr-FR --text "Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera." --out target/verify-eval/text
echo "==> UI typecheck / tests / Biome"
cd ui
bun install --frozen-lockfile
bun run typecheck
bun test
bun run check
cd "$ROOT"
echo "==> bridge build + smoke + 10 s delivery"
cargo build -q -p ll-bridge --release
cp target/release/liblivelinguist_core.so tests/bridge/livelinguist-core.node
node tests/bridge/bridge-smoke.mjs
node tests/bridge/event-delivery.mjs 10
bun tests/bridge/event-delivery.mjs 10
echo "==> pass"
