#!/usr/bin/env bash
# Privacy check, Linux dev (T35). The Windows version is privacy-check.ps1.
#
# Runs ll-eval with the real models on a known speech clip (sherpa ASR, llama
# simplifier, every segment clicked), then checks:
#   1. content-free outputs: no reference word appears in any output file or
#      in the run's stdout/stderr, except the explicitly content-bearing
#      exports (transcript.md, segments.csv, simplification.txt, hyp.txt);
#   2. no network: strace records every connect(); any AF_INET/AF_INET6
#      connect fails the check (models are already downloaded);
#   3. not vacuous: the content-bearing transcript must contain the words.
# Prints `privacy: PASS`, or the failing file/connection. Never prints the
# matched words.
#
# Usage: scripts/privacy-check.sh [--no-build] [CLIP.wav REFERENCE.txt]
# Defaults: target/asr-smoke/en.wav + target/asr-smoke/en.ref.txt (one FLEURS
# en_us test clip, CC-BY-4.0, never committed). Models: LL_MODELS_DIR or the
# default cache (see `cargo run -p ll-models --example fetch_models`).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BUILD=1
if [ "${1:-}" = "--no-build" ]; then BUILD=0; shift; fi
CLIP="${1:-target/asr-smoke/en.wav}"
REF="${2:-target/asr-smoke/en.ref.txt}"
OUT="target/privacy"
RUNLOG="target/privacy-run.log"
TRACE="target/privacy-strace.log"
for tool in strace python3; do
  command -v "$tool" >/dev/null || { echo "privacy: FAIL (missing tool: $tool)"; exit 2; }
done
[ -f "$CLIP" ] && [ -f "$REF" ] || { echo "privacy: FAIL (clip or reference missing)"; exit 2; }
if [ "$BUILD" = 1 ]; then
  cargo build -q --release -p ll-eval --features sherpa,llama
fi
rm -rf "$OUT" "$RUNLOG" "$TRACE"
strace -f -qq -e trace=connect -o "$TRACE" \
  target/release/ll-eval --scenario mock --language en-US --asr sherpa --llm llama \
  --wav "$CLIP" --click-all --out "$OUT" >"$RUNLOG" 2>&1 || {
  echo "privacy: FAIL (ll-eval exited non-zero; see $RUNLOG)"; exit 1; }

python3 -I - "$OUT" "$RUNLOG" "$TRACE" "$REF" <<'PY'
import os, re, sys
out, runlog, trace, ref = sys.argv[1:5]
CONTENT_BEARING = {"transcript.md", "segments.csv", "simplification.txt", "hyp.txt"}
words = sorted({w.strip(".,;:!?\"'").lower() for w in open(ref, encoding="utf-8").read().split()},
               key=len, reverse=True)
words = [w for w in words if w.isalpha() and len(w) >= 6][:5]
if len(words) < 3:
    print("privacy: FAIL (reference has too few distinctive words)"); sys.exit(1)
failures = []
def scan(path, label):
    text = open(path, encoding="utf-8", errors="replace").read().lower()
    for index, word in enumerate(words):
        if re.search(r"\b" + re.escape(word) + r"\b", text):
            failures.append(f"{label}: reference word #{index + 1}")
found_in_transcript = 0
for folder, _, files in os.walk(out):
    for name in files:
        path = os.path.join(folder, name)
        if name in CONTENT_BEARING:
            text = open(path, encoding="utf-8", errors="replace").read().lower()
            found_in_transcript += sum(1 for w in words if w in text)
            continue
        scan(path, os.path.relpath(path))
scan(runlog, "ll-eval stdout/stderr")
for line in open(trace, encoding="utf-8", errors="replace"):
    if "AF_INET" in line:
        family = "AF_INET6" if "AF_INET6" in line else "AF_INET"
        failures.append(f"network: connect() with {family}")
if found_in_transcript == 0:
    failures.append("vacuous: the transcript export has none of the reference words")
if failures:
    for failure in sorted(set(failures)):
        print("privacy: FAIL", failure)
    sys.exit(1)
files = sum(len(f) for _, _, f in os.walk(out))
print(f"checked {files} files + run log; {len(words)} reference words; connects traced")
print("privacy: PASS")
PY
