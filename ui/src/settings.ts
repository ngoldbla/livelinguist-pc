import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";

/** Preferences the user can change in Console and Settings. */
export interface UiSettings {
  captionSize: number;
  showTier1: boolean;
  detection: boolean;
  devEnabled: boolean;
  scenario: string;
}

export const CAPTION_SIZE_MIN = 16;
export const CAPTION_SIZE_MAX = 40;

export const defaultSettings: UiSettings = {
  captionSize: 24,
  showTier1: true,
  detection: true,
  devEnabled: true,
  scenario: "mock",
};

/** `%LOCALAPPDATA%\LiveLinguist\ui-settings.json`, else the XDG config dir. */
export function settingsPath(): string {
  const base =
    process.platform === "win32"
      ? (process.env.LOCALAPPDATA ?? join(homedir(), "AppData", "Local"))
      : (process.env.XDG_CONFIG_HOME ?? join(homedir(), ".config"));
  return join(base, "LiveLinguist", "ui-settings.json");
}

function clampSize(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return defaultSettings.captionSize;
  return Math.max(CAPTION_SIZE_MIN, Math.min(CAPTION_SIZE_MAX, Math.round(value)));
}

function bool(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

/** Read saved preferences; anything missing or malformed keeps its default. */
export function loadSettings(path: string = settingsPath()): UiSettings {
  try {
    const raw = JSON.parse(readFileSync(path, "utf8")) as Partial<UiSettings> | null;
    if (!raw || typeof raw !== "object") return defaultSettings;
    return {
      captionSize: clampSize(raw.captionSize),
      showTier1: bool(raw.showTier1, defaultSettings.showTier1),
      detection: bool(raw.detection, defaultSettings.detection),
      devEnabled: bool(raw.devEnabled, defaultSettings.devEnabled),
      scenario:
        typeof raw.scenario === "string" && raw.scenario ? raw.scenario : defaultSettings.scenario,
    };
  } catch {
    return defaultSettings;
  }
}

/** Persist preferences. Failures are silent: the session keeps working. */
export function saveSettings(settings: UiSettings, path: string = settingsPath()): boolean {
  try {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, `${JSON.stringify(settings, null, 2)}\n`, "utf8");
    return true;
  } catch {
    return false;
  }
}
