// Packaged (bun build --compile) layout, ADR 0021 option 1: livelinguist.exe
// with gpuix-native.win32-x64-msvc.node and livelinguist-core.node beside it.
// Pure functions so the layout is tested on Linux.

export const GPUIX_ADDON = "gpuix-native.win32-x64-msvc.node";
export const CORE_ADDON = "livelinguist-core.node";

/** Bun's virtual file system root for a compiled executable's modules. */
export function isCompiledMain(mainPath: string): boolean {
  return (
    mainPath.startsWith("B:\\~BUN\\") ||
    mainPath.startsWith("B:/~BUN/") ||
    mainPath.startsWith("/$bunfs/")
  );
}

/** Directory of an executable path, for either separator style. */
export function exeDir(execPath: string): string {
  const cut = Math.max(execPath.lastIndexOf("\\"), execPath.lastIndexOf("/"));
  return cut < 0 ? "." : execPath.slice(0, cut);
}

function join(dir: string, file: string): string {
  const sep = dir.includes("\\") ? "\\" : "/";
  return `${dir}${sep}${file}`;
}

export interface PackagedPaths {
  dir: string;
  gpuixAddon: string;
  coreAddon: string;
}

export function packagedPaths(execPath: string): PackagedPaths {
  const dir = exeDir(execPath);
  return { dir, gpuixAddon: join(dir, GPUIX_ADDON), coreAddon: join(dir, CORE_ADDON) };
}

function mainPath(): string {
  const bun = (globalThis as { Bun?: { main?: string } }).Bun;
  return bun?.main ?? "";
}

/** True when running from the compiled, shipped executable. */
export function isPackaged(): boolean {
  return isCompiledMain(mainPath());
}

/**
 * Point the GPUIX napi loader at the addon beside the exe. Must run before
 * `@gpuix/*` is first imported (main.tsx imports it dynamically afterwards).
 */
export function preparePackagedEnvironment(env: Record<string, string | undefined>): void {
  if (!isPackaged()) return;
  if (!env.NAPI_RS_NATIVE_LIBRARY_PATH) {
    env.NAPI_RS_NATIVE_LIBRARY_PATH = packagedPaths(process.execPath).gpuixAddon;
  }
}
