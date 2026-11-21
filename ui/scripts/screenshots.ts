#!/usr/bin/env bun
// Real-window screenshots for the Linux dev portfolio.
//
// GPUIX 0.11 has no Linux PNG test renderer, so this launches the real X11
// window and captures it with ImageMagick `import -window`. On Wayland, run
// under XWayland with LL_FORCE_X11=1.

import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";

const ROOT = resolve(import.meta.dir, "../..");
const UI = resolve(ROOT, "ui");
const OUT = resolve(ROOT, "docs/screenshots/m2");
const DISPLAY = process.env.DISPLAY ?? ":0";
const wait = (ms: number) => new Promise((resolveWait) => setTimeout(resolveWait, ms));

const addon = resolve(UI, "livelinguist-core.node");
if (!existsSync(addon)) {
  console.log("building ll-bridge addon…");
  const build = Bun.spawnSync(["cargo", "build", "-p", "ll-bridge", "--release"], {
    cwd: ROOT,
    stdout: "inherit",
    stderr: "inherit",
  });
  if (build.exitCode !== 0) throw new Error("cargo build failed");
  copyFileSync(resolve(ROOT, "target/release/liblivelinguist_core.so"), addon);
}

mkdirSync(OUT, { recursive: true });

const scenarios = [
  { name: "onboarding-welcome", screen: "onboarding", wait: 3500 },
  {
    name: "onboarding-language",
    screen: "onboarding",
    extra: { LL_ONBOARDING_BEAT: "1" },
    wait: 3500,
  },
  {
    name: "onboarding-models",
    screen: "onboarding",
    extra: { LL_ONBOARDING_BEAT: "2" },
    wait: 3500,
  },
  { name: "console", screen: "console", wait: 4000 },
  {
    name: "console-arabic",
    screen: "console",
    extra: { LL_LANGUAGE: "ar-SA", LL_UI_LOCALE: "en" },
    wait: 4000,
  },
  { name: "live", screen: "live", extra: { LL_MODE: "live" }, wait: 8000 },
  { name: "expanded", screen: "expanded", extra: { LL_MODE: "live" }, wait: 8000 },
  { name: "overlay", screen: "overlay", extra: { LL_MODE: "live" }, wait: 8000 },
  { name: "settings", screen: "settings", wait: 4000 },
  { name: "settings-developer", screen: "settings", extra: { LL_TAB: "developer" }, wait: 4000 },
  { name: "summary", screen: "summary", extra: { LL_MODE: "live" }, wait: 9000 },
  { name: "licenses", screen: "licenses", wait: 3500 },
] as const;

function findWindow(pid: number): string | null {
  const search = Bun.spawnSync(["xdotool", "search", "--name", "Live Linguist"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const ids = new TextDecoder().decode(search.stdout).trim().split("\n").filter(Boolean);
  for (const id of ids.reverse()) {
    const prop = Bun.spawnSync(["xprop", "-id", id, "_NET_WM_PID"], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const text = new TextDecoder().decode(prop.stdout);
    if (text.includes(`= ${pid}`)) return id;
  }
  return null;
}

for (const scenario of scenarios) {
  console.log(`\n▶ ${scenario.name}`);
  const proc = Bun.spawn(["bun", "src/main.tsx"], {
    cwd: UI,
    env: {
      ...process.env,
      DISPLAY,
      LL_FORCE_X11: "1",
      GPUIX_BACKGROUND: "1",
      LL_SCREEN: scenario.screen,
      LL_LANGUAGE: "fr-FR",
      LL_UI_LOCALE: "en",
      ...("extra" in scenario ? scenario.extra : {}),
    },
    stdout: "pipe",
    stderr: "pipe",
  });
  let windowId: string | null = null;
  const deadline = Date.now() + 20_000;
  while (!windowId && Date.now() < deadline) {
    await wait(400);
    windowId = findWindow(proc.pid);
  }
  if (!windowId) {
    const err = await new Response(proc.stderr).text();
    console.error(`no window for ${scenario.name}: ${err.slice(-500)}`);
    proc.kill();
    continue;
  }
  await wait(scenario.wait);
  const target = resolve(OUT, `${scenario.name}.png`);
  const capture = Bun.spawnSync(["import", "-display", DISPLAY, "-window", windowId, target], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if (capture.exitCode !== 0) {
    console.error(new TextDecoder().decode(capture.stderr));
  } else {
    console.log(`  → ${target}`);
  }
  proc.kill();
  await wait(800);
}
console.log("\nscreenshots complete");
