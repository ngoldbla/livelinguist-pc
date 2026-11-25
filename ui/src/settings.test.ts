import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  CAPTION_SIZE_MAX,
  CAPTION_SIZE_MIN,
  defaultSettings,
  loadSettings,
  saveSettings,
  settingsPath,
} from "./settings";

function tempPath(): string {
  return join(mkdtempSync(join(tmpdir(), "ll-settings-")), "ui-settings.json");
}

describe("ui settings", () => {
  test("a missing file returns the defaults", () => {
    const path = tempPath();
    expect(loadSettings(path)).toEqual(defaultSettings);
    rmSync(join(path, ".."), { recursive: true, force: true });
  });

  test("a saved round trip restores every preference", () => {
    const path = tempPath();
    const saved = {
      captionSize: 26,
      showTier1: false,
      detection: false,
      reducedEffects: true,
      followLearning: false,
      devEnabled: false,
      scenario: "cpu-strict",
    };
    expect(saveSettings(saved, path)).toBe(true);
    expect(loadSettings(path)).toEqual(saved);
    rmSync(join(path, ".."), { recursive: true, force: true });
  });

  test("caption size is clamped and bad types fall back", () => {
    const path = tempPath();
    writeFileSync(
      path,
      JSON.stringify({
        captionSize: 99,
        showTier1: "yes",
        detection: 1,
        scenario: "",
      }),
    );
    const loaded = loadSettings(path);
    expect(loaded.captionSize).toBe(CAPTION_SIZE_MAX);
    expect(loaded.showTier1).toBe(defaultSettings.showTier1);
    expect(loaded.detection).toBe(defaultSettings.detection);
    expect(loaded.scenario).toBe(defaultSettings.scenario);

    writeFileSync(path, JSON.stringify({ captionSize: 1 }));
    expect(loadSettings(path).captionSize).toBe(CAPTION_SIZE_MIN);
    rmSync(join(path, ".."), { recursive: true, force: true });
  });

  test("junk JSON does not throw", () => {
    const path = tempPath();
    writeFileSync(path, "{ not json");
    expect(loadSettings(path)).toEqual(defaultSettings);
    rmSync(join(path, ".."), { recursive: true, force: true });
  });

  test("the settings path ends in the product directory", () => {
    expect(settingsPath().replace(/\\/g, "/")).toMatch(/LiveLinguist\/ui-settings\.json$/);
  });
});
