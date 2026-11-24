// Fit window sizes to the monitor work area so nothing clips at 150% scale.

export interface WorkArea {
  width: number;
  height: number;
  scale: number;
}

export interface Size {
  width: number;
  height: number;
}

export const MIN_LOGICAL: Size = { width: 960, height: 620 };

/** Overlay keeps its fixed size; only console/expanded windows are fitted. */
export const OVERLAY_SIZE: Size = { width: 780, height: 250 };

/**
 * Clamp `desired` (logical pixels) to the logical work area minus `margin`.
 * Never goes below `MIN_LOGICAL` unless the screen itself is smaller, in
 * which case the available size wins. `area == null` returns `desired`.
 */
export function fitWindow(desired: Size, area: WorkArea | null, margin = 24): Size {
  if (!area || !(area.width > 0) || !(area.height > 0)) return desired;
  const scale = area.scale > 0 ? area.scale : 1;
  const available = {
    width: Math.floor(area.width / scale - margin),
    height: Math.floor(area.height / scale - margin),
  };
  const axis = (want: number, room: number, min: number): number => {
    if (room < min) return room;
    return Math.max(min, Math.min(want, room));
  };
  return {
    width: axis(desired.width, available.width, MIN_LOGICAL.width),
    height: axis(desired.height, available.height, MIN_LOGICAL.height),
  };
}

/** Parse the bridge's `workArea()` JSON; anything unexpected → null. */
export function parseWorkArea(json: string | null | undefined): WorkArea | null {
  if (!json) return null;
  try {
    const value = JSON.parse(json) as Partial<WorkArea> | null;
    if (
      !value ||
      typeof value.width !== "number" ||
      typeof value.height !== "number" ||
      typeof value.scale !== "number"
    ) {
      return null;
    }
    return { width: value.width, height: value.height, scale: value.scale };
  } catch {
    return null;
  }
}

export type WindowModeName = "console" | "overlay" | "expanded";

/** Size a mode requests, fitted to the work area. Overlay stays 780x250. */
export function sizeForMode(mode: WindowModeName, area: WorkArea | null): Size {
  if (mode === "overlay") return OVERLAY_SIZE;
  const desired = mode === "expanded" ? { width: 1180, height: 900 } : { width: 1180, height: 820 };
  return fitWindow(desired, area);
}
