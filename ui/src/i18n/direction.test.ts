import { describe, expect, test } from "bun:test";
import { directionFor, rowDirection, textAlign } from "./direction";

describe("direction helpers", () => {
  test("Arabic and Hebrew are RTL", () => {
    expect(directionFor("ar")).toBe("rtl");
    expect(directionFor("ar-SA")).toBe("rtl");
    expect(directionFor("he")).toBe("rtl");
    expect(rowDirection("ar")).toBe("row-reverse");
    expect(textAlign("ar")).toBe("right");
  });

  test("Latin and CJK are LTR", () => {
    expect(directionFor("fr")).toBe("ltr");
    expect(directionFor("ja-JP")).toBe("ltr");
    expect(rowDirection("ko")).toBe("row");
    expect(textAlign("de")).toBe("left");
  });
});
