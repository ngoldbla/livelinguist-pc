import { describe, expect, test } from "bun:test";
import type { RuntimeSupport } from "../bridge/generated/RuntimeSupport";
import type { UiRendererStatus } from "../bridge/generated/UiRendererStatus";
import { acceleratorLabel, rendererAdapter, rendererLabel } from "./renderer";

function ui(patch: Partial<UiRendererStatus>): UiRendererStatus {
  return {
    backend: "directx",
    apiAvailable: true,
    hardware: false,
    adapter: null,
    driver: null,
    degradedReason: null,
    notes: [],
    ...patch,
  };
}

describe("renderer status labels", () => {
  test("unknown is never presented as a GPU", () => {
    expect(rendererLabel(null)).toBe("not probed");
    expect(rendererLabel(ui({ backend: "unknown", hardware: false }))).toBe("not probed");
    expect(rendererLabel(ui({ backend: "demo", hardware: false }))).toBe("demo · software");
  });

  test("a measured hardware adapter is labelled GPU", () => {
    expect(
      rendererLabel(
        ui({ backend: "directx", hardware: true, adapter: "NVIDIA GeForce RTX 5060 Ti" }),
      ),
    ).toBe("directx · GPU");
  });

  test("the adapter falls back to the measured degraded reason", () => {
    expect(rendererAdapter(ui({ adapter: "NVIDIA GeForce RTX 5060 Ti" }))).toBe(
      "NVIDIA GeForce RTX 5060 Ti",
    );
    expect(rendererAdapter(ui({ degradedReason: "only a software adapter is available" }))).toBe(
      "only a software adapter is available",
    );
    expect(rendererAdapter(null)).toBe("unknown");
  });
});

describe("accelerator labels", () => {
  const cuda = (patch: Partial<RuntimeSupport>): RuntimeSupport => ({
    available: true,
    detail: "CUDA driver 13.5",
    devices: [],
    ...patch,
  });

  test("reports device, VRAM and compute capability", () => {
    expect(
      acceleratorLabel(
        cuda({
          devices: [{ name: "NVIDIA GeForce RTX 5060 Ti", vramTotalMb: 16283, compute: "12.0" }],
        }),
      ),
    ).toBe("NVIDIA GeForce RTX 5060 Ti · 16283 MB · cc 12.0");
  });

  test("availability without a device is not a device claim", () => {
    expect(acceleratorLabel(cuda({ devices: [] }))).toBe("available");
    expect(acceleratorLabel(cuda({ available: false, devices: [] }))).toBe("unavailable");
    expect(acceleratorLabel(null)).toBe("not probed");
  });
});
