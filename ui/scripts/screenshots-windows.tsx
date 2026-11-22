#!/usr/bin/env bun
// Windows UI portfolio through the GPUIX GPU test renderer.
//
// This renders the real React app offscreen with GPUIX's native DirectX
// renderer and captures PNGs. It requires an interactive desktop session: a
// process in session 0 (a service) can enumerate DXGI adapters but cannot
// create the DirectX window, so run it through
// `scripts/run-in-interactive-session.ps1` on a dev VM whose agent runs in
// session 0. It never writes transcript text; a11y output is counts only.

import { mkdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const ROOT = resolve(import.meta.dir, "../..");
const OUT = resolve(ROOT, "docs/screenshots/m2-windows");
const FONTS = ["C:\\Windows\\Fonts\\segoeui.ttf", "C:\\Windows\\Fonts\\tahoma.ttf"];

interface Scenario {
  name: string;
  screen: string;
  mode?: "live";
  env?: Record<string, string>;
  size?: [number, number];
  waitMs?: number;
}

const scenarios: Scenario[] = [
  { name: "onboarding-welcome", screen: "onboarding", waitMs: 1200 },
  {
    name: "onboarding-language",
    screen: "onboarding",
    env: { LL_ONBOARDING_BEAT: "1" },
    waitMs: 1200,
  },
  {
    name: "onboarding-models",
    screen: "onboarding",
    env: { LL_ONBOARDING_BEAT: "2" },
    waitMs: 1200,
  },
  { name: "console", screen: "console", waitMs: 1500 },
  {
    name: "console-arabic",
    screen: "console",
    env: { LL_LANGUAGE: "ar-SA", LL_UI_LOCALE: "en" },
    waitMs: 1500,
  },
  { name: "live", screen: "live", mode: "live", waitMs: 2500 },
  { name: "expanded", screen: "expanded", mode: "live", size: [1180, 900], waitMs: 2500 },
  {
    name: "overlay",
    screen: "overlay",
    mode: "live",
    size: [780, 250],
    waitMs: 2500,
  },
  { name: "settings", screen: "settings", size: [1180, 880], waitMs: 1500 },
  {
    name: "settings-developer",
    screen: "settings",
    env: { LL_TAB: "developer" },
    size: [1180, 880],
    waitMs: 1500,
  },
  { name: "summary", screen: "summary", mode: "live", size: [1180, 840], waitMs: 2500 },
  { name: "licenses", screen: "licenses", size: [1020, 880], waitMs: 1200 },
];

const sleep = (ms: number) => new Promise((done) => setTimeout(done, ms));

const only = new Set(
  (process.argv.find((arg) => arg.startsWith("--only=")) ?? "")
    .replace("--only=", "")
    .split(",")
    .filter(Boolean),
);
const skip = new Set(
  (process.argv.find((arg) => arg.startsWith("--skip=")) ?? "")
    .replace("--skip=", "")
    .split(",")
    .filter(Boolean),
);

function setScenarioEnv(scenario: Scenario): void {
  process.env.LL_SCREEN = scenario.screen;
  process.env.LL_LANGUAGE = "fr-FR";
  process.env.LL_UI_LOCALE = "en";
  process.env.LL_MODE = scenario.mode ?? "on_demand";
  delete process.env.LL_ONBOARDING_BEAT;
  delete process.env.LL_TAB;
  for (const [key, value] of Object.entries(scenario.env ?? {})) {
    process.env[key] = value;
  }
}

interface A11ySummary {
  nodes: number;
  roles: Record<string, number>;
  unlabeledControls: number;
}

function summarizeA11y(tree: unknown): A11ySummary {
  const nodes = Object.values(
    (tree as { nodes?: Record<string, { aria?: { role?: string; label?: string } }> }).nodes ??
      {},
  );
  const roles: Record<string, number> = {};
  let unlabeledControls = 0;
  const controls = new Set(["button", "checkbox", "switch", "slider", "textbox", "link", "tab"]);
  for (const node of nodes) {
    const role = node.aria?.role;
    if (!role) continue;
    roles[role] = (roles[role] ?? 0) + 1;
    if (controls.has(role) && !node.aria?.label) unlabeledControls += 1;
  }
  return { nodes: nodes.length, roles, unlabeledControls };
}

const { createTestRoot, hasNativeTestRenderer } = await import("@gpuix/react/testing");
if (!hasNativeTestRenderer) {
  console.error("This GPUIX build has no native test renderer. Use the real-window path.");
  process.exit(2);
}

const { App } = await import("../src/App");
mkdirSync(OUT, { recursive: true });

let failures = 0;
for (const scenario of scenarios) {
  if (only.size > 0 && !only.has(scenario.name)) continue;
  if (skip.has(scenario.name)) continue;
  setScenarioEnv(scenario);
  const [width, height] = scenario.size ?? [1180, 820];
  let root: ReturnType<typeof createTestRoot> | null = null;
  try {
    root = createTestRoot({ width, height });
  } catch (error) {
    console.error(
      `GPUIX DirectX renderer could not start: ${(error as Error).message}\n` +
        "This needs an interactive desktop session (not session 0). " +
        "Run: pwsh -File scripts/run-in-interactive-session.ps1 " +
        "-FilePath bun -Arguments scripts/screenshots-windows.tsx -WorkingDirectory ui",
    );
    process.exit(2);
  }

  try {
    for (const font of FONTS) {
      try {
        root.renderer.registerFont(readFileSync(font));
      } catch {
        // A missing optional font is not fatal.
      }
    }
    root.render(<App />);
    const waitMs = scenario.waitMs ?? 1200;
    for (let elapsed = 0; elapsed < waitMs; elapsed += 250) {
      await sleep(250);
      root.renderer.flush();
    }
    root.renderer.flush();
    const windowSize = root.renderer.getWindowSize();
    let keyboardStops = 0;
    if (scenario.name === "console") {
      const stops = new Set<number>();
      for (let step = 0; step < 4; step += 1) {
        root.renderer.simulateKeystrokes("tab");
        root.renderer.flush();
        const focused = root.renderer.getFocusedElementId();
        if (focused == null) break;
        stops.add(focused);
      }
      keyboardStops = stops.size;
    }
    const target = join(OUT, `${scenario.name}.png`);
    root.renderer.captureScreenshot(target);
    const a11y = summarizeA11y(root.renderer.getA11yTree());
    const roles = Object.entries(a11y.roles)
      .map(([role, count]) => `${role}:${count}`)
      .join(" ");
    console.log(
      `  ${scenario.name}.png  ${windowSize.width}x${windowSize.height}  a11y nodes=${a11y.nodes} unlabeled=${a11y.unlabeledControls} keyboard=${keyboardStops} ${roles}`,
    );
    if (a11y.unlabeledControls > 0) failures += 1;
    if (scenario.name === "console" && keyboardStops < 2) {
      failures += 1;
      console.error("  console: keyboard focus did not move through tab stops");
    }
    if (scenario.name === "console") {
      const startButton = root.renderer.findByText("Start captions");
      const bounds = startButton ? root.renderer.getElementBounds(startButton.id) : null;
      if (!bounds) {
        failures += 1;
        console.error("  console: Start captions button not found");
      } else {
        root.renderer.nativeSimulateClick(
          bounds.x + bounds.width / 2,
          bounds.y + bounds.height / 2,
        );
        for (let elapsed = 0; elapsed < 2000; elapsed += 250) {
          await sleep(250);
          root.renderer.flush();
        }
        root.renderer.flush();
        root.renderer.captureScreenshot(join(OUT, "console-start-captions.png"));
        const liveText = root.renderer.getAllText().join(" ");
        const started = liveText.includes("CAPTIONS");
        console.log(
          `  console-start-captions.png  click path ${started ? "reached the live screen" : "did not reach live"}`,
        );
        if (!started) {
          failures += 1;
          console.error("  console: Start captions click did not open the live screen");
        }
      }
    }
  } catch (error) {
    failures += 1;
    console.error(`  ${scenario.name}: capture failed: ${(error as Error).message}`);
  } finally {
    root.unmount();
    await sleep(250);
  }
}

if (failures > 0) {
  console.error(`\n${failures} scenario(s) need attention`);
  process.exit(1);
}
console.log(`\nscreenshots complete: ${OUT}`);
process.exit(0);
