import type { RuntimeSupport } from "../bridge/generated/RuntimeSupport";
import type { UiRendererStatus } from "../bridge/generated/UiRendererStatus";

/**
 * Human-readable renderer summary. It never upgrades "unknown" or "demo" into
 * a GPU claim: the value comes from a measured DXGI/DRM probe.
 */
export function rendererLabel(ui: UiRendererStatus | null | undefined): string {
  if (!ui || ui.backend === "unknown") return "not probed";
  return `${ui.backend} · ${ui.hardware ? "GPU" : "software"}`;
}

/** The adapter name, or the measured reason it is not usable. */
export function rendererAdapter(ui: UiRendererStatus | null | undefined): string {
  if (!ui) return "unknown";
  if (ui.adapter) return ui.adapter;
  return ui.degradedReason ?? "no adapter reported";
}

/** Accelerator device names from a runtime probe (CUDA today). */
export function acceleratorLabel(runtime: RuntimeSupport | null | undefined): string {
  if (!runtime) return "not probed";
  if (runtime.devices.length > 0) {
    return runtime.devices
      .map((device) => {
        const vram = device.vramTotalMb > 0 ? ` · ${device.vramTotalMb} MB` : "";
        const compute = device.compute ? ` · cc ${device.compute}` : "";
        return `${device.name}${vram}${compute}`;
      })
      .join(", ");
  }
  return runtime.available ? "available" : "unavailable";
}
