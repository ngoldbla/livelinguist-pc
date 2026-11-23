# T24 — Size windows to the work area (S6 layout)

**Needs:** T23 done.

**Files:** create `ui/src/windowSize.ts` + `ui/src/windowSize.test.ts`,
edit `ui/src/main.tsx`, and the place that handles `setWindowMode`
(search `setWindowMode` in `ui/src`).

## Steps

1. `windowSize.ts`:

   ```ts
   export interface WorkArea { width: number; height: number; scale: number }
   export const MIN_LOGICAL = { width: 960, height: 620 };
   export function fitWindow(desired: {width:number;height:number}, area: WorkArea | null,
     margin = 24): {width:number;height:number}
   ```

   Logical available = `area.width / area.scale - margin`, same for height.
   Result = `min(desired, available)` per axis, but never below `MIN_LOGICAL`
   (if the screen is smaller than the minimum, return the available size).
   `area == null` → return `desired`.
2. Tests: 1920x1080 @150% with taskbar (work area 1920x1032) and desired
   1180x900 → height ≤ 664; 1280x720 @100% (work area 1280x672) → height ≤ 648;
   2560x1080 @100% → desired unchanged; null → desired.
3. `main.tsx`: before `render`, call the addon's `workArea()` (guard with
   try/catch; demo transport → null), and pass `fitWindow(...)` sizes. Keep
   `LL_WIDTH`/`LL_HEIGHT` env overrides as they are.
4. On mode switch to `console`/`expanded`, apply the same fit to the size the
   mode requests. Overlay stays 780x250.

## Done when

```powershell
cd ui; bun run typecheck; bun test; bun run check; cd ..
```

Commit message: `ui: fit windows to the monitor work area`
