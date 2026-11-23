# T38 — Clean-profile install test (S8)

**Needs:** T37 done. Needs one admin action to create a test user.

**Goal:** prove install/upgrade/uninstall work for a normal user with no admin
rights and no dev tools, on this PC (no VM).

## Steps

1. From an admin PowerShell:
   `net user llclean <a password> /add` (standard user, not admin). Sign out,
   sign in as `llclean`. Copy `dist\` to `C:\Users\Public\llclean-test\`
   first so the user can reach it.
2. As `llclean`: run the setup. **There must be no UAC prompt.** Launch,
   onboarding → download models (~1.08 GB) → System audio with a YouTube
   video → captions → click to simplify → overlay → export → close.
3. Upgrade: install a build with a bumped patch version over it; settings and
   models remain; app launches.
4. Uninstall (leave the delete-models box off) → app gone, models remain;
   reinstall → no re-download. Uninstall again with the box on → models gone.
5. Portable: unzip to the Desktop, run, works (models found or downloaded).
6. Check: `reg query HKLM\Software /s /f LiveLinguist` finds nothing; files only
   under `%LOCALAPPDATA%\Programs\LiveLinguist` and `%LOCALAPPDATA%\LiveLinguist`.
7. As admin afterwards: `net user llclean /delete` and delete its profile
   folder.

## Done when

Every step recorded pass/fail with screenshots in
`docs/qa/2026-11-23-packaging-v1.md` `## Clean profile` and
`docs/screenshots/m2-windows/install/`. Any failure → fix in this card or
`BLOCKED.md`.

Commit message: `qa: clean-profile install, upgrade, uninstall`
