// Keyboard logic for the caption list (T27). Pure functions so they are
// unit-tested without a window; the screens only wire them to GPUIX events.

export type CaptionAction = "move" | "simplify" | "none";

export interface CaptionKeyResult {
  index: number;
  action: CaptionAction;
}

export interface KeyModifiers {
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  cmd?: boolean;
}

const UP = new Set(["arrow-up", "up", "arrowup"]);
const DOWN = new Set(["arrow-down", "down", "arrowdown"]);
const ACTIVATE = new Set(["enter", "space", " "]);

/** Current row + key → next focused row and what to do. */
export function captionKeyAction(index: number, count: number, key: string): CaptionKeyResult {
  if (count <= 0) return { index: -1, action: "none" };
  const current = Math.max(0, Math.min(index, count - 1));
  const name = key.toLowerCase();
  if (UP.has(name)) return { index: Math.max(0, current - 1), action: "move" };
  if (DOWN.has(name)) return { index: Math.min(count - 1, current + 1), action: "move" };
  if (name === "home") return { index: 0, action: "move" };
  if (name === "end") return { index: count - 1, action: "move" };
  if (ACTIVATE.has(name)) return { index: current, action: "simplify" };
  return { index: current, action: "none" };
}

/** Ctrl+Shift+L: Read latest. */
export function isReadLatestShortcut(key: string, modifiers: KeyModifiers | undefined): boolean {
  return key.toLowerCase() === "l" && Boolean(modifiers?.ctrl) && Boolean(modifiers?.shift);
}

export const READ_LATEST_SHORTCUT = "Ctrl+Shift+L";

/** Accessible name for a caption row: sentence text plus its state. */
export function captionAccessibleLabel(
  text: string,
  simplified: string | null,
  simplifying: boolean,
): string {
  if (simplified) return `${text}. Simplified: ${simplified}`;
  if (simplifying) return `${text}. Simplifying`;
  return `${text}. Click to simplify`;
}

/** Index of the row "Read latest" focuses: newest simplified, else newest. */
export function latestCaptionIndex(segments: ReadonlyArray<{ simplified: string | null }>): number {
  for (let index = segments.length - 1; index >= 0; index -= 1) {
    if (segments[index]?.simplified) return index;
  }
  return segments.length - 1;
}

type WindowKeyListener = (key: string, modifiers: KeyModifiers | undefined) => boolean;

const listeners = new Set<WindowKeyListener>();

/**
 * Window-level key bus: `main.tsx` forwards GPUIX window `onKeyDown` here and
 * screens subscribe while mounted. Returns true when a listener handled it.
 */
export const windowKeys = {
  subscribe(listener: WindowKeyListener): () => void {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
  dispatch(key: string, modifiers: KeyModifiers | undefined): boolean {
    let handled = false;
    for (const listener of listeners) {
      if (listener(key, modifiers)) handled = true;
    }
    return handled;
  },
};
