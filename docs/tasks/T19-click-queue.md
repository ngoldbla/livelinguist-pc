# T19 — Click-only queue rules (S5)

**Needs:** T18 done. Works with mock backends; no model needed for tests.

**Goal:** simplification happens only when the user clicks, with at most one
running and one waiting.

**Files:** whichever of `core/crates/ll-pipeline/src/lib.rs` /
`core/crates/ll-llm/src/lib.rs` already holds the simplify queue (search for
`queue`, `queued`, `cancel_simplify`). Bridge only if a signature changes.

## Rules to enforce (write a test for each first, then make them pass)

1. A click on segment A starts A immediately if nothing runs.
2. While A runs, a click on B queues B. A click on C replaces B (B is dropped
   and gets a "superseded" result, not an error banner).
3. `cancel_simplify(A)` while A runs stops it (the backend gets a cancel flag
   or its result is discarded) and starts the queued item if any.
   `cancel_simplify(B)` while queued removes it.
4. Clicking a segment that is already simplified returns the cached result and
   does not call the backend.
5. **No background work:** feeding audio, partials, and finalized segments for
   60 s of demo audio with no clicks makes **zero** backend calls (count calls
   in `MockBackend`). Live/automatic mode must not exist on the click path in
   1.0: if `SimplificationMode::Automatic` still triggers calls, gate it behind
   the dev-only scenario and document that in the test name.

## Done when

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Commit message: `pipeline: click-only queue rules`
