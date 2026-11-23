# T16 — Onboarding download screen (first-run UX) (PR)

**Needs:** T15 done.

**Goal:** the "Download models" button really downloads, shows progress, can
be cancelled and retried, and the app knows when models are ready.

**Files:** `ui/src/bridge/transport.ts`, `ui/src/screens/Onboarding.tsx`,
`ui/src/screens/Settings.tsx` (model status row), `ui/src/store/sessionStore.ts`
(if progress lives in the store), tests next to them.

## Steps

1. `transport.ts`: add to `Transport`:
   `modelStatus(): Promise<ModelStatus[]>`, `downloadModels(): Promise<void>`,
   `cancelDownload(): Promise<boolean>`. Native transport calls the T15
   functions; the demo transport returns both models as `present: false` and
   `downloadModels` rejects with "not available in demo mode".
2. Route `model_progress` / `model_ready` / `model_error` envelopes to state
   (follow how other envelope kinds reach the store; respect the `seq` rule).
3. `Onboarding.tsx` (the `beat === 2` block): replace the toast-only button.
   - If both models are present: show "Models ready" and a **Continue** button.
   - Else **Download models (~1.08 GB)** starts the download; show one progress
     bar per model (percent + MB), a **Cancel** button, and on error the reason
     plus **Retry**. Keep "Continue in demo mode".
   - Every new control has a visible label (the a11y check counts unlabeled
     controls).
4. `Settings.tsx`: a row per model with present/missing and a **Download**
   button when missing.
5. Tests (`bun test`): store reducer handles progress → ready, progress →
   error → retry, and ignores envelopes with an old `seq`.

## Done when

```powershell
cd ui; bun run typecheck; bun test; bun run check; cd ..
```

pass, and a manual run with the native addon: delete the model folder (or
point `LOCALAPPDATA` to a temp dir), download from onboarding, cancel halfway,
retry (it resumes), finish → "Models ready". Screenshots:
`docs/screenshots/m2-windows/onboarding-download-progress.png`,
`onboarding-models-ready.png`.

Open PR `windows: first-run model download` from branch
`stage-w3-model-download` (T15–T16), merge when CI is green.

Commit message: `ui: first-run model download`
