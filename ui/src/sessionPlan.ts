import { join } from "node:path";
import type { AudioSource } from "./store/sessionStore";

export type ExportFormat = "markdown" | "plain" | "csv";

/// What the bridge is asked to start for the source the user picked.
/// `feedDemo` is true only for the demo source: system audio and files must
/// never be padded with scripted demo speech.
export type SessionPlan =
  | { ok: true; source: AudioSource; feedDemo: boolean; asrStatus: "simplify_only" | "tbd" }
  | { ok: false; error: string };

export function planSession(selected: string): SessionPlan {
  switch (selected) {
    case "demo":
      return { ok: true, source: "demo", feedDemo: true, asrStatus: "simplify_only" };
    case "monitor":
      return { ok: true, source: "monitor", feedDemo: false, asrStatus: "tbd" };
    case "silence":
    case "file":
      return {
        ok: true,
        source: selected as AudioSource,
        feedDemo: false,
        asrStatus: "simplify_only",
      };
    case "mic":
      return { ok: false, error: "Microphone input is not available in this version." };
    default:
      return { ok: false, error: `Unknown audio source: ${selected}` };
  }
}

export function exportExtension(format: ExportFormat): string {
  return format === "markdown" ? "md" : format === "plain" ? "txt" : "csv";
}

/// The folder exports go to: `<home>/Documents/Live Linguist`.
export function exportDir(home: string): string {
  return join(home, "Documents", "Live Linguist");
}

/// A file name that is valid on Windows (no `:`), sortable, and unique per second.
export function exportPath(format: ExportFormat, now: Date, home: string): string {
  const stamp = now.toISOString().slice(0, 19).replace(/[:T]/g, "-");
  return join(exportDir(home), `livelinguist-${stamp}.${exportExtension(format)}`);
}
