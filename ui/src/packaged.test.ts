import { describe, expect, test } from "bun:test";
import { CORE_ADDON, exeDir, GPUIX_ADDON, isCompiledMain, packagedPaths } from "./packaged";

describe("packaged layout (ADR 0021)", () => {
  test("recognises Bun's compiled virtual file system", () => {
    expect(isCompiledMain("B:\\~BUN\\root\\livelinguist.exe")).toBe(true);
    expect(isCompiledMain("B:/~BUN/root/main.js")).toBe(true);
    expect(isCompiledMain("/$bunfs/root/livelinguist")).toBe(true);
    expect(isCompiledMain("C:\\src\\ui\\src\\main.tsx")).toBe(false);
    expect(isCompiledMain("/home/dev/ui/src/main.tsx")).toBe(false);
    expect(isCompiledMain("")).toBe(false);
  });

  test("addons resolve beside the installed exe", () => {
    const paths = packagedPaths(
      "C:\\Users\\someone\\AppData\\Local\\Programs\\LiveLinguist\\livelinguist.exe",
    );
    expect(paths.dir).toBe("C:\\Users\\someone\\AppData\\Local\\Programs\\LiveLinguist");
    expect(paths.gpuixAddon).toBe(`${paths.dir}\\${GPUIX_ADDON}`);
    expect(paths.coreAddon).toBe(`${paths.dir}\\${CORE_ADDON}`);
  });

  test("posix and bare paths", () => {
    expect(packagedPaths("/opt/ll/livelinguist").coreAddon).toBe(`/opt/ll/${CORE_ADDON}`);
    expect(exeDir("livelinguist.exe")).toBe(".");
  });

  test("addon names match what the build script copies", async () => {
    const script = await Bun.file(`${import.meta.dir}/../../scripts/build-windows.ps1`).text();
    expect(script).toContain(GPUIX_ADDON);
    expect(script).toContain(CORE_ADDON);
  });
});
