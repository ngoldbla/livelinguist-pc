# T21 — Tier C profile, click latency, and RSS (S5)

Linux part done in PR #31; Windows pending: run `scripts\tier-c-profile.ps1` click-bench (threads 3) on the reference PC and add the row.

**Needs:** T20 done.

**Goal:** measure click-to-simplified p50/p95 and memory under a low-end CPU
profile. Gate: **p95 ≤ 8 s, peak RSS ≤ 3.5 GB**.

**Files:** create `scripts/tier-c-profile.ps1`; `core/crates/ll-eval/src/lib.rs`

## Steps

1. `scripts/tier-c-profile.ps1 -- <exe> <args...>`: starts the process with
   `Start-Process -PassThru`, immediately sets `$p.ProcessorAffinity = 0xF`
   (4 logical CPUs) and `$p.PriorityClass = 'BelowNormal'`, waits, samples
   `$p.PeakWorkingSet64` every second, and prints `peak_rss_mb=<n>` at the end.
   Header comment: "Tier C approximation: 4 logical CPUs on the reference host;
   memory is measured, not limited."
2. `ll-eval`: scenario `--scenario click-bench --clicks <n>`: uses the
   built-in demo conversation for the language, finalizes its segments, then
   "clicks" each one in turn through the same pipeline path the UI uses, and
   records click → simplified ms. Summary adds `click_p50_ms`, `click_p95_ms`,
   `clicks`, `threads`. No text in the summary.
3. Build release, then run 50 clicks for en-US under the profile with
   `--threads 2` and again with `--threads 3`:

   ```powershell
   cargo build --release -p ll-eval --features llama,sherpa
   pwsh scripts\tier-c-profile.ps1 -- target\release\ll-eval.exe --scenario click-bench --clicks 50 --language en-US --llm llama --threads 3 --out target\bench-c3
   ```

   Also once without the profile (tier A reference).
4. If p95 > 8 s: try `--threads 2`, and a smaller max-token cap in
   `LlamaBackend` once. Do not add GPU. Record every attempt.

## Done when

`docs/qa/2026-11-23-llm-cpu.md` has `## Tier C` with a table
(threads, p50, p95, peak RSS, pass/fail) and the chosen default thread count,
and that default is set in `ll-config`. Unit test for `click-bench` argument
parsing passes.

Commit message: `eval: click benchmark and tier C profile`
