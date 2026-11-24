import { describe, expect, test } from "bun:test";
import {
  captionAccessibleLabel,
  captionKeyAction,
  isReadLatestShortcut,
  latestCaptionIndex,
  windowKeys,
} from "./captionKeys";

describe("captionKeyAction", () => {
  test("down and up move within bounds", () => {
    expect(captionKeyAction(0, 3, "arrow-down")).toEqual({ index: 1, action: "move" });
    expect(captionKeyAction(2, 3, "arrow-down")).toEqual({ index: 2, action: "move" });
    expect(captionKeyAction(1, 3, "arrow-up")).toEqual({ index: 0, action: "move" });
    expect(captionKeyAction(0, 3, "arrow-up")).toEqual({ index: 0, action: "move" });
  });

  test("home and end jump", () => {
    expect(captionKeyAction(1, 5, "home")).toEqual({ index: 0, action: "move" });
    expect(captionKeyAction(1, 5, "end")).toEqual({ index: 4, action: "move" });
  });

  test("enter and space simplify the current row", () => {
    expect(captionKeyAction(2, 3, "enter")).toEqual({ index: 2, action: "simplify" });
    expect(captionKeyAction(1, 3, "space")).toEqual({ index: 1, action: "simplify" });
    expect(captionKeyAction(1, 3, " ")).toEqual({ index: 1, action: "simplify" });
  });

  test("other keys do nothing; empty list has no row", () => {
    expect(captionKeyAction(1, 3, "a")).toEqual({ index: 1, action: "none" });
    expect(captionKeyAction(0, 0, "enter")).toEqual({ index: -1, action: "none" });
  });

  test("a stale index is clamped", () => {
    expect(captionKeyAction(9, 3, "enter")).toEqual({ index: 2, action: "simplify" });
  });
});

describe("read latest", () => {
  test("Ctrl+Shift+L only", () => {
    expect(isReadLatestShortcut("l", { ctrl: true, shift: true })).toBe(true);
    expect(isReadLatestShortcut("L", { ctrl: true, shift: true })).toBe(true);
    expect(isReadLatestShortcut("l", { ctrl: true })).toBe(false);
    expect(isReadLatestShortcut("k", { ctrl: true, shift: true })).toBe(false);
    expect(isReadLatestShortcut("l", undefined)).toBe(false);
  });

  test("focuses the newest simplified row, else the newest row", () => {
    expect(latestCaptionIndex([{ simplified: "a" }, { simplified: null }])).toBe(0);
    expect(latestCaptionIndex([{ simplified: null }, { simplified: null }])).toBe(1);
    expect(latestCaptionIndex([])).toBe(-1);
  });

  test("window key bus reaches subscribers until they unsubscribe", () => {
    let calls = 0;
    const off = windowKeys.subscribe((key, modifiers) => {
      if (isReadLatestShortcut(key, modifiers)) {
        calls += 1;
        return true;
      }
      return false;
    });
    expect(windowKeys.dispatch("l", { ctrl: true, shift: true })).toBe(true);
    expect(windowKeys.dispatch("x", {})).toBe(false);
    off();
    expect(windowKeys.dispatch("l", { ctrl: true, shift: true })).toBe(false);
    expect(calls).toBe(1);
  });
});

describe("captionAccessibleLabel", () => {
  test("states", () => {
    expect(captionAccessibleLabel("Hello there", null, false)).toBe(
      "Hello there. Click to simplify",
    );
    expect(captionAccessibleLabel("Hello there", null, true)).toBe("Hello there. Simplifying");
    expect(captionAccessibleLabel("Hello there", "Hi", false)).toBe("Hello there. Simplified: Hi");
  });
});
