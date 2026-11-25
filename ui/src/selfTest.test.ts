import { describe, expect, test } from "bun:test";
import { selfTestArg } from "./selfTest";

describe("self-test flag", () => {
  test("parses the flag and an optional report path", () => {
    expect(selfTestArg(["livelinguist.exe"])).toEqual({ enabled: false });
    expect(selfTestArg(["livelinguist.exe", "--self-test"])).toEqual({ enabled: true });
    expect(selfTestArg(["x", "--self-test=C:\\t\\r.json"])).toEqual({
      enabled: true,
      outPath: "C:\\t\\r.json",
    });
  });
});
