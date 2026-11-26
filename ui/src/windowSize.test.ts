import { describe, expect, test } from "bun:test";
import { fitWindow, MIN_LOGICAL, OVERLAY_SIZE, parseWorkArea, sizeForMode } from "./windowSize";

describe("fitWindow", () => {
  test("1920x1080 at 150% with taskbar fits the expanded window", () => {
    const size = fitWindow({ width: 1180, height: 900 }, { width: 1920, height: 1032, scale: 1.5 });
    expect(size.height).toBeLessThanOrEqual(664);
    expect(size.width).toBeLessThanOrEqual(1280 - 24);
    expect(size.height).toBeGreaterThanOrEqual(MIN_LOGICAL.height);
  });

  test("1280x720 at 100% with taskbar", () => {
    const size = fitWindow({ width: 1180, height: 900 }, { width: 1280, height: 672, scale: 1 });
    expect(size.height).toBeLessThanOrEqual(648);
    expect(size.width).toBe(1180);
  });

  test("2560x1080 at 100% leaves the desired size unchanged", () => {
    const desired = { width: 1180, height: 820 };
    expect(fitWindow(desired, { width: 2560, height: 1032, scale: 1 })).toEqual(desired);
  });

  test("null work area returns desired", () => {
    const desired = { width: 1180, height: 900 };
    expect(fitWindow(desired, null)).toEqual(desired);
  });

  test("never below the minimum when the screen has room", () => {
    const size = fitWindow({ width: 400, height: 300 }, { width: 1920, height: 1032, scale: 1 });
    expect(size).toEqual(MIN_LOGICAL);
  });

  test("screen smaller than the minimum returns the available size", () => {
    const size = fitWindow({ width: 1180, height: 900 }, { width: 1024, height: 600, scale: 1 });
    expect(size).toEqual({ width: 1000, height: 576 });
  });
});

describe("parseWorkArea", () => {
  test("parses bridge JSON", () => {
    expect(parseWorkArea('{"x":0,"y":0,"width":1920,"height":1032,"scale":1.5}')).toEqual({
      width: 1920,
      height: 1032,
      scale: 1.5,
    });
  });

  test("null and junk are null", () => {
    expect(parseWorkArea("null")).toBeNull();
    expect(parseWorkArea(null)).toBeNull();
    expect(parseWorkArea("{")).toBeNull();
    expect(parseWorkArea('{"width":"x"}')).toBeNull();
  });
});

describe("sizeForMode", () => {
  const area = { width: 1920, height: 1032, scale: 1.5 };
  test("overlay keeps 780x250", () => {
    expect(sizeForMode("overlay", area)).toEqual(OVERLAY_SIZE);
  });
  test("console and expanded are fitted", () => {
    expect(sizeForMode("expanded", area).height).toBeLessThanOrEqual(664);
    expect(sizeForMode("console", area).height).toBeLessThanOrEqual(664);
    expect(sizeForMode("console", null)).toEqual({ width: 960, height: 660 });
  });
});
