import { preparePackagedEnvironment } from "./packaged";

// Launcher/bootstrap environment. The launcher/AppRun owns this in production;
// this is the documented fallback for `bun src/app.tsx`.
export function prepareEnvironment(): void {
  preparePackagedEnvironment(process.env);
  const forceX11 = process.env.LL_FORCE_X11 === "1" || process.env.LL_SCREEN === "overlay";
  if (forceX11) {
    delete process.env.WAYLAND_DISPLAY;
  }
}
