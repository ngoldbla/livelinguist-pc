import { describe, expect, test } from "bun:test";
import { join } from "node:path";
import { exportDir, exportPath, planSession } from "./sessionPlan";

describe("planSession", () => {
  test("system audio is passed through and never fed demo speech", () => {
    expect(planSession("monitor")).toEqual({
      ok: true,
      source: "monitor",
      feedDemo: false,
      asrStatus: "tbd",
    });
  });

  test("only the demo source is fed the scripted conversation", () => {
    for (const source of ["monitor", "silence", "file"]) {
      const plan = planSession(source);
      expect(plan.ok && plan.feedDemo).toBe(false);
    }
    const demo = planSession("demo");
    expect(demo.ok && demo.feedDemo).toBe(true);
  });

  test("microphone is refused, not swapped for demo", () => {
    const plan = planSession("mic");
    expect(plan.ok).toBe(false);
  });

  test("unknown sources are refused", () => {
    expect(planSession("speaker").ok).toBe(false);
  });
});

describe("exportPath", () => {
  const now = new Date("2026-11-24T09:05:07.123Z");

  test("goes to Documents/Live Linguist under the home folder, never /tmp", () => {
    const path = exportPath("markdown", now, "C:\\Users\\someone");
    expect(path.startsWith(exportDir("C:\\Users\\someone"))).toBe(true);
    expect(path).not.toContain("/tmp");
    expect(exportDir("/home/a")).toBe(join("/home/a", "Documents", "Live Linguist"));
  });

  test("file name has no colon and the format's extension", () => {
    const name = exportPath("plain", now, "/home/a").split(/[\\/]/).pop();
    expect(name).toBe("livelinguist-2026-11-24-09-05-07.txt");
    expect(exportPath("csv", now, "/home/a").endsWith(".csv")).toBe(true);
    expect(exportPath("markdown", now, "/home/a").endsWith(".md")).toBe(true);
  });
});
