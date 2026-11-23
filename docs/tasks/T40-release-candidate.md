# T40 — Release candidate and `v1.0.0` (S9)

**Needs:** S8 merged.

## Steps

1. Fill `docs/qa/2026-11-23-v1-release-gates.md`: one row per gate G0–G9 from
   `docs/plans/2026-11-22-v1-scope.md` §5, each with pass/fail and a link to
   its evidence file. Gaps that are allowed non-claims (for example full RTL
   mirroring) are listed as such.
2. If **any** gate fails: write `BLOCKED.md` with the gate and stop.
3. If all pass: bump the workspace version to `1.0.0`, update
   `README.md` Status, the progress board (S9 merged), and ADR statuses
   if any changed. PR `release: v1.0.0`; merge when CI is green.
4. Tag `v1.0.0` on the merge commit and push the tag (the tag name is approved
   by the owner). Confirm the release has setup, portable ZIP, and
   `SHA256SUMS.txt`.

## Done when

The `v1.0.0` release exists with its three assets, and the gates file shows
G0–G9 green.

Commit message: `release: v1.0.0`
