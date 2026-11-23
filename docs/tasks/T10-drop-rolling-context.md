# T10 — Remove rolling context from the click path (S4)

**Needs:** T09 done.

**Goal:** when the user clicks a sentence, only that sentence goes to the
simplifier. No previous sentences.

**Files:** `core/crates/ll-pipeline/src/lib.rs` (and its tests)

## Steps

1. In `ll-pipeline/src/lib.rs` find the click path (search `let previous = match self`
   inside the function that simplifies one segment by id). Replace the block
   that collects the two previous segments with an empty `Vec::new()` and
   delete the now-unused code.
2. Find the live path (search `let previous = self.context.clone();`). Also
   pass an empty `Vec`. If `self.context` is then unused, remove the field and
   the code that fills it.
3. Do not change `ll-core` prompt functions or their goldens in this card.
4. Add a pipeline test: create a session with 3 finalized segments, click the
   third, and assert the backend received a request whose user prompt does not
   contain the text of segments 1 and 2. Use the existing `MockBackend` (it
   receives the `SimplificationRequest`); if it doesn't store requests, add a
   `last_request` field to it.
5. Run `rg "previous" core/crates/ll-pipeline/src/lib.rs` and make sure no
   remaining match feeds earlier segments into a simplify call.

## Done when

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Then open a PR `core: EasyLanguage prompt contract` from the branch holding
T08–T10 (branch `stage-w4-prompt-contract`), and merge it when CI is green.
Set S4 to `merged` in `docs/qa/implementation-progress.md`.

Commit message: `pipeline: no rolling context on the click path`
