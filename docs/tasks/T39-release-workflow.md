# T39 — Release workflow dry run + SmartScreen note (S8) (PR)

**Needs:** T38 done.

**Files:** `.github/workflows/release.yml`, `README.md`, `docs/qa/2026-11-23-packaging-v1.md`

## Steps

1. `release.yml`: on tag `v*`, build on `windows-latest` with the real
   features, run `build-windows.ps1 -Package`, upload setup, portable ZIP,
   and `SHA256SUMS.txt` to the GitHub release. No signing step yet (ADR 0020);
   leave a commented placeholder "SignPath signing step goes here once the
   project is accepted". No secrets.
2. Dry run: push tag `v0.9.0-rc.1`; confirm the workflow attaches the three
   files; download them and check the hashes match. Delete the test release and
   tag afterwards (GitHub UI or `gh release delete`).
3. README: add a "Install on Windows" section: download, verify SHA-256
   (`Get-FileHash`), SmartScreen "More info → Run anyway" (unsigned for now),
   no admin needed, models download on first run (~1.08 GB), nothing leaves the
   device.

## Done when

Dry-run release had all three assets with matching hashes. Open PR
`windows: installer and release` from `stage-w6-packaging` (T37–T39); merge
when CI is green; set S8 to `merged`.

Commit message: `release: tag workflow and install notes`
