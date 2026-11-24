# T27 — Click-to-simplify from the keyboard + "Read latest" (S6 a11y)

Linux part done in PR #26; Windows pending: keyboard-only live run (start session, move to caption, simplify, Read latest), `verify-windows.ps1 -A11y`, focus ring at 100/125/150%.

**Needs:** T26 done.

**Files:** `ui/src/screens/Live.tsx`, `ui/src/screens/Console.tsx`,
`ui/src/aurora/components.tsx` (only if the caption row component lives there)

## Steps

1. Each caption sentence row must be focusable and activatable: Tab reaches the
   caption list, Up/Down move between sentences, Enter or Space triggers the
   same simplify call as a mouse click. The focused sentence has a visible
   focus ring at 100/125/150%.
2. The row's accessible label is the sentence text (this is UI, not logs, so
   text is allowed here) plus state: "simplified", "simplifying", or
   "click to simplify".
3. "Read latest" (already in `Live.tsx`): give it a keyboard shortcut
   (Ctrl+Shift+L) shown in its label. If GPUIX exposes a way to announce text
   to screen readers, use it; otherwise move focus to the latest caption row so
   Narrator reads it, and write that in the a11y doc as the method.
4. `bun test`: a unit test for the Up/Down/Enter handler logic (pure function:
   current index + key → new index / action).

## Done when

UI checks pass; `verify-windows.ps1 -A11y` still passes; a keyboard-only run
(no mouse) can start a session, move to a caption, simplify it, and use Read
latest. Note the result in `docs/qa/2026-11-23-v1-ui.md` under `## Keyboard`.

Commit message: `ui: keyboard simplify and read-latest`
