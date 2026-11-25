// `livelinguist.exe --self-test`: load both native addons without opening a
// window, then exit 0 (ok) or 1. Release CI runs it on the packaged and the
// installed app. Output is content-free: names, versions, and error messages.
import { writeFileSync } from "node:fs";
import { selfTestCore } from "./bridge/transport";
import { isPackaged } from "./packaged";

export interface SelfTestReport {
  ok: boolean;
  packaged: boolean;
  gpuix: string;
  core: string;
}

export async function runSelfTest(outPath?: string): Promise<number> {
  const report: SelfTestReport = { ok: false, packaged: isPackaged(), gpuix: "", core: "" };
  try {
    const native = (await import("@gpuix/native")) as { __napiBindingTarget?: string };
    const target = native.__napiBindingTarget ?? "unknown";
    report.gpuix = target === "native" ? "ok" : `unexpected binding: ${target}`;
  } catch (error) {
    report.gpuix = `failed: ${error instanceof Error ? error.message : String(error)}`;
  }
  try {
    selfTestCore();
    report.core = "ok";
  } catch (error) {
    report.core = `failed: ${error instanceof Error ? error.message : String(error)}`;
  }
  report.ok = report.gpuix === "ok" && report.core === "ok";
  const line = JSON.stringify(report);
  console.log(line);
  if (outPath) writeFileSync(outPath, `${line}\n`);
  return report.ok ? 0 : 1;
}

/** `--self-test` or `--self-test=<report.json>` from argv. */
export function selfTestArg(argv: string[]): { enabled: boolean; outPath?: string } {
  for (const arg of argv) {
    if (arg === "--self-test") return { enabled: true };
    if (arg.startsWith("--self-test=")) return { enabled: true, outPath: arg.slice(12) };
  }
  return { enabled: false };
}
