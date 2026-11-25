import { createHash } from "node:crypto";
import { existsSync, readdirSync, writeFileSync } from "node:fs";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import {
  alignCaptions,
  type CaptionCue,
  type CaptionObservation,
  type MediaClockSample,
  MediaClock,
  parseSrt,
  percentile,
  srtToVtt,
  tokenize,
} from "./align";

interface FixtureManifest {
  id: string;
  title: string;
  author: string;
  license: string;
  media: { file: string; url: string; sha256: string; bytes: number; durationSeconds: number };
  captions: { file: string; url: string; sha256: string; language: string };
  benchmarkWindow: { startSeconds: number; seconds: number };
  /** App language for a spoken fixture; defaults to en-US. */
  language?: string;
  /** App UI locale; defaults to en so the UIA probe's labels stay stable. */
  uiLocale?: string;
}

interface ClickEvent {
  index: number;
  t: number;
  ok: boolean;
  inWindow: boolean;
}

interface SimplifiedEvent {
  index: number;
  t: number;
}

interface BenchmarkOptions {
  app: string;
  sourceUi: boolean;
  mode: "live" | "overlay";
  seconds: number;
  repeat: number;
  startSeconds: number;
  asrChunkMs?: number;
  asrThreads?: number;
  asrRule1Ms?: number;
  asrRule2Ms?: number;
  asrRule3Ms?: number;
  out: string;
  fixture: string;
  fixtureDir: string;
  pollMs: number;
  clickAtMs: number;
  clickEveryN: number;
  record?: string;
  ffmpeg?: string;
  commit?: string;
}

interface TrialMedia {
  clock: MediaClockSample[];
  start: MediaClockSample | null;
  stop: MediaClockSample | null;
  error: string | null;
}

interface TrialResult {
  schemaVersion: number;
  fixtureId: string;
  trial: number;
  app: { path: string; sha256: string };
  commit: string;
  config: {
    mode: string;
    startSeconds: number;
    seconds: number;
    asrChunkMs: number | null;
    asrThreads: number | null;
    asrRule1Ms: number | null;
    asrRule2Ms: number | null;
    asrRule3Ms: number | null;
    pollMs: number;
    clickEveryN: number;
  };
  media: {
    tickCount: number;
    startWallMs: number | null;
    stopWallMs: number | null;
    stopMediaMs: number | null;
    pollQuantumP95Ms: number | null;
    error: string | null;
  };
  alignment: ReturnType<typeof alignCaptions>["summary"];
  cues: ReturnType<typeof alignCaptions>["matches"];
  click: {
    requested: boolean;
    app: number;
    ok: boolean;
    maxGapMs: number | null;
    mediaMs: number | null;
  } | null;
  clickSeries: {
    requested: number;
    clicked: number;
    simplified: number;
    failed: number;
    maxGapMs: number | null;
    p50Ms: number | null;
    p95Ms: number | null;
    events: Array<{
      index: number;
      mediaMs: number | null;
      latencyMs: number | null;
      ok: boolean;
    }>;
  } | null;
  recording: {
    file: string;
    seconds: number;
    width: number;
    height: number;
    fps: number;
    trimMs: number;
    mediaStartWallMs: number;
    firstFrameWallMs: number;
    audioSource: string;
  } | null;
  valid: boolean;
  contentFree: true;
}

const ROOT = resolve(import.meta.dir, "..");
const FIXTURE_PATH = join(ROOT, "autoresearch", "fixture.json");
const PLAYER_PATH = join(ROOT, "autoresearch", "player.html");
const UIA_PATH = join(ROOT, "autoresearch", "uia-captions.ps1");

function argument(name: string): string | null {
  const index = process.argv.indexOf(name);
  return index >= 0 ? (process.argv[index + 1] ?? null) : null;
}

function numberArgument(name: string, fallback: number): number {
  const value = argument(name);
  if (value === null) return fallback;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) throw new Error(`${name} needs a number`);
  return parsed;
}

function optionalNumberArgument(name: string): number | undefined {
  const value = argument(name);
  if (value === null) return undefined;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) throw new Error(`${name} needs a number`);
  return parsed;
}

function defaultApp(): string {
  const candidate = join(ROOT, "dist", "windows", "livelinguist.exe");
  if (existsSync(candidate)) return candidate;
  const installed = join(
    process.env.LOCALAPPDATA ?? "",
    "Programs",
    "LiveLinguist",
    "livelinguist.exe",
  );
  if (!existsSync(installed)) throw new Error(`no app found; pass --app <path>`);
  return installed;
}

async function options(): Promise<BenchmarkOptions> {
  const mode = argument("--mode") ?? "live";
  if (mode !== "live" && mode !== "overlay") throw new Error("--mode must be live or overlay");
  const sourceUi = process.argv.includes("--source-ui");
  const asrChunkMs = optionalNumberArgument("--asr-chunk-ms");
  const asrThreads = optionalNumberArgument("--asr-threads");
  const asrRule1Ms = optionalNumberArgument("--asr-rule1-ms");
  const asrRule2Ms = optionalNumberArgument("--asr-rule2-ms");
  const asrRule3Ms = optionalNumberArgument("--asr-rule3-ms");
  if (asrChunkMs !== undefined && (asrChunkMs < 80 || asrChunkMs > 2000)) {
    throw new Error("--asr-chunk-ms must be 80..2000");
  }
  if (asrThreads !== undefined && (asrThreads < 1 || asrThreads > 64)) {
    throw new Error("--asr-threads must be 1..64");
  }
  for (const [name, value] of [
    ["--asr-rule1-ms", asrRule1Ms],
    ["--asr-rule2-ms", asrRule2Ms],
  ] as const) {
    if (value !== undefined && (value < 100 || value > 10_000)) {
      throw new Error(`${name} must be 100..10000`);
    }
  }
  if (asrRule3Ms !== undefined && (asrRule3Ms < 1000 || asrRule3Ms > 60_000)) {
    throw new Error("--asr-rule3-ms must be 1000..60000");
  }
  const clickEveryN = numberArgument("--click-every-n", 0);
  if (clickEveryN < 0 || clickEveryN > 100) {
    throw new Error("--click-every-n must be 0..100");
  }
  return {
    app: resolve(
      argument("--app") ?? (sourceUi ? join(ROOT, "ui", "src", "main.tsx") : defaultApp()),
    ),
    sourceUi,
    mode,
    seconds: numberArgument("--seconds", 90),
    repeat: numberArgument("--repeat", 1),
    startSeconds: numberArgument("--start-seconds", 0),
    asrChunkMs,
    asrThreads,
    asrRule1Ms,
    asrRule2Ms,
    asrRule3Ms,
    out: resolve(argument("--out") ?? join(ROOT, "target", "autoresearch", "runs", stamp())),
    fixture: resolve(argument("--fixture") ?? FIXTURE_PATH),
    fixtureDir: resolve(argument("--fixture-dir") ?? join(ROOT, "target", "autoresearch", "media")),
    pollMs: numberArgument("--poll-ms", 20),
    clickAtMs: numberArgument("--click-at-ms", 0),
    clickEveryN,
    record: argument("--record") ? resolve(argument("--record") as string) : undefined,
    ffmpeg: argument("--ffmpeg") ?? process.env.FFMPEG ?? undefined,
    commit: argument("--commit") ?? undefined,
  };
}

function stamp(): string {
  return new Date().toISOString().replace(/[:.]/g, "-");
}

function assertContentFree(value: unknown, path = "$"): void {
  if (Array.isArray(value)) {
    value.forEach((item, index) => assertContentFree(item, `${path}[${index}]`));
    return;
  }
  if (value === null || typeof value !== "object") return;
  for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
    if (["text", "hypothesis", "transcript", "audio", "video", "uia"].includes(key)) {
      throw new Error(`content-bearing field in benchmark output: ${path}.${key}`);
    }
    assertContentFree(child, `${path}.${key}`);
  }
}

async function sha256(path: string): Promise<string> {
  return createHash("sha256").update(await readFile(path)).digest("hex");
}

async function ensureFixture(manifest: FixtureManifest, dir: string): Promise<void> {
  await mkdir(dir, { recursive: true });
  for (const item of [manifest.media, manifest.captions]) {
    const path = join(dir, item.file);
    if (existsSync(path) && (await sha256(path)) === item.sha256) continue;
    const response = await fetch(item.url);
    if (!response.ok) throw new Error(`fixture download failed: ${response.status}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    const digest = createHash("sha256").update(bytes).digest("hex");
    if (digest !== item.sha256) throw new Error(`fixture hash mismatch: ${item.file}`);
    await writeFile(path, bytes);
  }
}

function contentType(path: string): string {
  if (path.endsWith(".webm")) return "video/webm";
  if (path.endsWith(".vtt")) return "text/vtt; charset=utf-8";
  return "text/html; charset=utf-8";
}

async function serveFile(path: string, request: Request): Promise<Response> {
  const file = Bun.file(path);
  const size = file.size;
  const range = request.headers.get("range");
  if (!range) return new Response(file, { headers: { "Content-Type": contentType(path) } });
  const match = /^bytes=(\d+)-(\d*)$/.exec(range);
  if (!match) {
    return new Response("invalid range", {
      status: 416,
      headers: { "Content-Range": `bytes */${size}` },
    });
  }
  const start = Number(match[1]);
  const end = match[2] ? Number(match[2]) : size - 1;
  if (start >= size || end < start) {
    return new Response("invalid range", {
      status: 416,
      headers: { "Content-Range": `bytes */${size}` },
    });
  }
  const safeEnd = Math.min(end, size - 1);
  return new Response(file.slice(start, safeEnd + 1), {
    status: 206,
    headers: {
      "Content-Type": contentType(path),
      "Accept-Ranges": "bytes",
      "Content-Range": `bytes ${start}-${safeEnd}/${size}`,
      "Content-Length": String(safeEnd - start + 1),
    },
  });
}

async function readLines(stream: ReadableStream<Uint8Array>, onLine: (line: string) => void) {
  const decoder = new TextDecoder();
  let buffer = "";
  for await (const chunk of stream) {
    buffer += decoder.decode(chunk, { stream: true });
    let newline = buffer.indexOf("\n");
    while (newline >= 0) {
      onLine(buffer.slice(0, newline).replace(/\r$/, ""));
      buffer = buffer.slice(newline + 1);
      newline = buffer.indexOf("\n");
    }
  }
  if (buffer.trim()) onLine(buffer.replace(/\r$/, ""));
}

async function commandOutput(args: string[]): Promise<string> {
  const proc = Bun.spawn(args, { stdout: "pipe", stderr: "ignore" });
  const output = await new Response(proc.stdout).text();
  await proc.exited;
  return output.trim();
}

async function killTree(pid: number): Promise<void> {
  const proc = Bun.spawn(["taskkill", "/PID", String(pid), "/T", "/F"], {
    stdout: "ignore",
    stderr: "ignore",
  });
  await proc.exited;
}

function edgePath(): string {
  const candidates = [
    join(process.env["ProgramFiles(x86)"] ?? "", "Microsoft", "Edge", "Application", "msedge.exe"),
    join(process.env.ProgramFiles ?? "", "Microsoft", "Edge", "Application", "msedge.exe"),
  ];
  const found = candidates.find((path) => existsSync(path));
  if (!found) throw new Error("Microsoft Edge was not found");
  return found;
}

/** Resolve ffmpeg: explicit flag, FFMPEG env, target/tools, then PATH. */
function ffmpegPath(explicit?: string): string {
  if (explicit) {
    if (!existsSync(explicit)) throw new Error(`ffmpeg not found: ${explicit}`);
    return explicit;
  }
  const local = join(ROOT, "target", "tools", "ffmpeg");
  if (existsSync(local)) {
    const found = readdirSync(local, { recursive: true })
      .map((entry) => join(local, String(entry)))
      .find((path) => path.toLowerCase().endsWith("ffmpeg.exe"));
    if (found) return found;
  }
  const probe = Bun.spawnSync(["where.exe", "ffmpeg"], { stdout: "pipe", stderr: "ignore" });
  const line = probe.stdout.toString().split(/\r?\n/).find((value) => value.trim());
  if (line) return line.trim();
  throw new Error("ffmpeg not found; pass --ffmpeg or install it under target/tools/ffmpeg");
}

function summarizeClicks(
  clicks: ClickEvent[],
  simplified: SimplifiedEvent[],
  clock: MediaClock,
  observations: Array<{ t: number }>,
): NonNullable<TrialResult["clickSeries"]> {
  const done = new Map<number, number>();
  for (const event of simplified) {
    if (!done.has(event.index) || (done.get(event.index) ?? 0) > event.t) {
      done.set(event.index, event.t);
    }
  }
  const latencies: number[] = [];
  const events = clicks.map((click) => {
    const finished = done.get(click.index);
    const latencyMs =
      click.ok && finished !== undefined && finished >= click.t ? finished - click.t : null;
    if (latencyMs !== null) latencies.push(latencyMs);
    return {
      index: click.index,
      mediaMs: clock.at(click.t),
      latencyMs,
      ok: click.ok,
    };
  });
  return {
    requested: clicks.length,
    clicked: clicks.filter((click) => click.ok).length,
    simplified: latencies.length,
    failed: clicks.filter((click) => !click.ok).length,
    maxGapMs: clickMaxGapMs(observations, clicks),
    p50Ms: percentile(latencies, 50),
    p95Ms: percentile(latencies, 95),
    events,
  };
}

function clickMaxGapMs(
  observations: Array<{ t: number }>,
  clicks: Array<{ t: number }>,
): number | null {
  let max: number | null = null;
  for (const click of clicks) {
    const times = observations
      .map((observation) => observation.t)
      .filter((t) => t >= click.t - 2000 && t <= click.t + 8000)
      .sort((a, b) => a - b);
    for (let index = 1; index < times.length; index += 1) {
      const gap = times[index] - times[index - 1];
      if (max === null || gap > max) max = gap;
    }
  }
  return max;
}

interface Recording {
  raw: string;
  final: string;
  proc: ReturnType<typeof Bun.spawn>;
  firstFrame: Promise<number>;
  width: number;
  height: number;
}

function startRecording(
  ffmpeg: string,
  raw: string,
  final: string,
  rect: { x: number; y: number; width: number; height: number },
  seconds: number,
): Recording {
  const even = (value: number) => Math.max(2, Math.floor(value / 2) * 2);
  const x = Math.max(0, even(rect.x));
  const y = Math.max(0, even(rect.y));
  const width = even(rect.width);
  const height = even(rect.height);
  const progressPath = `${raw}.progress.txt`;
  // Truncate synchronously before ffmpeg opens it; a stale progress file would
  // resolve the first-frame promise immediately.
  writeFileSync(progressPath, "");
  const proc = Bun.spawn(
    [
      ffmpeg,
      "-y",
      "-hide_banner",
      "-loglevel",
      "warning",
      "-nostats",
      "-progress",
      progressPath,
      "-f",
      "gdigrab",
      "-framerate",
      "20",
      "-draw_mouse",
      "1",
      "-offset_x",
      String(x),
      "-offset_y",
      String(y),
      "-video_size",
      `${width}x${height}`,
      "-i",
      "desktop",
      "-t",
      String(seconds + 15),
      "-an",
      "-c:v",
      "libx264",
      "-preset",
      "ultrafast",
      "-crf",
      "20",
      "-pix_fmt",
      "yuv420p",
      raw,
    ],
    { stdout: "ignore", stderr: "pipe", stdin: "pipe" },
  );
  const firstFrame = (async () => {
    for (let attempt = 0; attempt < 100; attempt += 1) {
      await new Promise((done) => setTimeout(done, 50));
      if (proc.exitCode !== null) break;
      try {
        const text = await readFile(progressPath, "utf8");
        if (text.includes("progress=continue") || text.includes("frame=")) {
          // The first progress block lands about 0.5 s in; the first frame was
          // captured a little earlier. A quarter second keeps the trim inside
          // one polling quantum.
          return Date.now() - 250;
        }
      } catch {
        // The progress file is created on the first write.
      }
    }
    throw new Error("screen capture did not start");
  })();
  void new Response(proc.stderr).text().catch(() => {});
  return { raw, final, proc, firstFrame, width, height };
}

async function stopRecording(recording: Recording): Promise<void> {
  try {
    recording.proc.stdin.write("q\n");
    recording.proc.stdin.end();
  } catch {
    // Fall through to the exit race; the capture may already be gone.
  }
  await Promise.race([recording.proc.exited, new Promise((done) => setTimeout(done, 10_000))]);
  if (recording.proc.exitCode === null) await killTree(recording.proc.pid).catch(() => {});
}

async function finalizeRecording(
  ffmpeg: string,
  recording: Recording,
  mediaPath: string,
  startSeconds: number,
  seconds: number,
  firstFrameWallMs: number,
  mediaStartWallMs: number,
): Promise<{ trimMs: number }> {
  const trimSeconds = Math.max(0, (mediaStartWallMs - firstFrameWallMs) / 1000);
  const proc = Bun.spawn(
    [
      ffmpeg,
      "-y",
      "-hide_banner",
      "-loglevel",
      "error",
      "-ss",
      trimSeconds.toFixed(3),
      "-i",
      recording.raw,
      "-ss",
      String(startSeconds),
      "-i",
      mediaPath,
      "-map",
      "0:v:0",
      "-map",
      "1:a:0",
      "-t",
      String(seconds),
      "-c:v",
      "libx264",
      "-preset",
      "ultrafast",
      "-crf",
      "20",
      "-pix_fmt",
      "yuv420p",
      "-c:a",
      "aac",
      "-b:a",
      "160k",
      "-movflags",
      "+faststart",
      recording.final,
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  const stderr = new Response(proc.stderr).text();
  await proc.exited;
  const message = (await stderr).trim();
  if (proc.exitCode !== 0) {
    throw new Error(`recording mux failed: ${message || proc.exitCode}`);
  }
  return { trimMs: Math.round(trimSeconds * 1000) };
}

async function runTrial(
  config: BenchmarkOptions,
  manifest: FixtureManifest,
  cues: CaptionCue[],
  playerHtml: string,
  vtt: string,
  trial: number,
): Promise<TrialResult> {
  const trialDir = join(config.out, `trial-${trial}`);
  await mkdir(trialDir, { recursive: true });
  const stopFile = join(trialDir, "stop");
  await rm(stopFile, { force: true }).catch(() => {});
  const trialId = `trial-${trial}-${Date.now()}`;
  const media: TrialMedia = { clock: [], start: null, stop: null, error: null };
  const debug = process.env.LL_BENCH_DEBUG === "1";
  let readyInfo: { title?: string; rect?: { x: number; y: number; width: number; height: number } } | null =
    null;
  let markReady: () => void = () => {};
  let markStop: () => void = () => {};
  const ready = new Promise<void>((done) => {
    markReady = done;
  });
  const stopped = new Promise<void>((done) => {
    markStop = done;
  });

  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    idleTimeout: 120,
    async fetch(request) {
      const url = new URL(request.url);
      if (debug) console.error(`request: ${request.method} ${url.pathname}`);
      if (url.pathname === "/player") {
        return new Response(playerHtml, { headers: { "Content-Type": "text/html; charset=utf-8" } });
      }
      if (url.pathname === "/media.webm") {
        return serveFile(join(config.fixtureDir, manifest.media.file), request);
      }
      if (url.pathname === "/captions.vtt") return new Response(vtt);
      if (request.method !== "POST") return new Response("not found", { status: 404 });
      const body = (await request.json().catch(() => ({}))) as Record<string, unknown>;
      if (url.pathname === "/ready") {
        markReady();
      } else if (url.pathname === "/media/start") {
        const sample = {
          wallMs: Number(body.wallMs),
          mediaMs: Number(body.mediaMs),
        };
        if (Number.isFinite(sample.wallMs) && Number.isFinite(sample.mediaMs)) {
          media.start = sample;
          media.clock.push(sample);
        }
      } else if (url.pathname === "/media/tick") {
        const sample = {
          wallMs: Number(body.wallMs),
          mediaMs: Number(body.mediaMs),
        };
        if (Number.isFinite(sample.wallMs) && Number.isFinite(sample.mediaMs)) {
          media.clock.push(sample);
        }
      } else if (url.pathname === "/media/stop") {
        const sample = {
          wallMs: Number(body.wallMs),
          mediaMs: Number(body.mediaMs),
        };
        if (Number.isFinite(sample.wallMs) && Number.isFinite(sample.mediaMs)) {
          media.stop = sample;
          media.clock.push(sample);
        }
        markStop();
      } else if (url.pathname === "/media/error") {
        media.error = "media playback failed";
        markStop();
      }
      return Response.json({ ok: true });
    },
  });

  const baseUrl = `http://127.0.0.1:${server.port}`;
  const env: Record<string, string> = {
    ...process.env,
    LL_SCREEN: "console",
    LL_LANGUAGE: manifest.language ?? "en-US",
    LL_UI_LOCALE: manifest.uiLocale ?? "en",
    LL_MODE: "on_demand",
  };
  delete env.GPUIX_BACKGROUND;
  if (config.asrChunkMs !== undefined) env.LL_ASR_CHUNK_MS = String(config.asrChunkMs);
  if (config.asrThreads !== undefined) env.LL_ASR_THREADS = String(config.asrThreads);
  if (config.asrRule1Ms !== undefined) env.LL_ASR_RULE1_MS = String(config.asrRule1Ms);
  if (config.asrRule2Ms !== undefined) env.LL_ASR_RULE2_MS = String(config.asrRule2Ms);
  if (config.asrRule3Ms !== undefined) env.LL_ASR_RULE3_MS = String(config.asrRule3Ms);

  const app = config.sourceUi
    ? Bun.spawn(["bun", "src/main.tsx"], {
        cwd: join(ROOT, "ui"),
        env,
        stdout: "ignore",
        stderr: debug ? "pipe" : "ignore",
      })
    : Bun.spawn([config.app], {
        cwd: dirname(config.app),
        env,
        stdout: "ignore",
        stderr: debug ? "pipe" : "ignore",
      });
  if (debug && app.stderr) {
    void readLines(app.stderr, (line) => console.error(`app: ${line}`)).catch(() => {});
  }
  const observations: Array<Omit<CaptionObservation, "mediaMs" | "tokens">> = [];
  const clickEvents: ClickEvent[] = [];
  const simplifiedEvents: SimplifiedEvent[] = [];
  let probeError: string | null = null;
  const probeArgs = [
    "pwsh",
    "-NoProfile",
    "-File",
    UIA_PATH,
    "-ProcessId",
    String(app.pid),
    "-Mode",
    config.mode,
    "-StopFile",
    stopFile,
    "-ReadyUrl",
    `${baseUrl}/ready`,
    "-PollMs",
    String(config.pollMs),
    "-ClickAtMs",
    String(config.clickAtMs),
    "-ClickEveryN",
    String(config.clickEveryN),
  ];
  if (config.record) {
    probeArgs.push("-PlaceX", "1300", "-PlaceY", "60");
  }
  const probe = Bun.spawn(probeArgs, { stdout: "pipe", stderr: "pipe" });
  const probeReader = readLines(probe.stdout, (line) => {
    if (!line.trim()) return;
    try {
      const value = JSON.parse(line) as {
        ready?: boolean;
        title?: string;
        rect?: { x: number; y: number; width: number; height: number };
        error?: string;
        t?: number;
        kind?: "partial" | "final" | "click" | "simplified" | "click_timeout";
        index?: number;
        text?: string;
        scan_ms?: number;
        ok?: boolean;
        in_window?: boolean;
      };
      if (value.error) probeError = "ui probe failed";
      if (value.ready) {
        readyInfo = { title: value.title, rect: value.rect };
        markReady();
      }
      if (value.t && value.kind === "click" && typeof value.index === "number") {
        clickEvents.push({
          index: value.index,
          t: value.t,
          ok: value.ok === true,
          inWindow: value.in_window !== false,
        });
      } else if (value.t && value.kind === "simplified" && typeof value.index === "number") {
        simplifiedEvents.push({ index: value.index, t: value.t });
      } else if (value.t && (value.kind === "partial" || value.kind === "final") && value.text) {
        observations.push({
          t: value.t,
          kind: value.kind,
          text: value.text,
          scanMs: value.scan_ms ?? 0,
        });
      }
    } catch {
      probeError = "ui probe returned invalid output";
    }
  });
  const probeStderr = new Response(probe.stderr).text();
  let edge: ReturnType<typeof Bun.spawn> | null = null;
  let recording: Recording | null = null;
  let firstFrameWallMs = 0;
  if (debug) console.error(`player: ${baseUrl}/player?start=${config.startSeconds}&seconds=${config.seconds}&trial=${trialId}`);

  try {
    await Promise.race([
      ready,
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("app did not become ready")), 45_000),
      ),
    ]);
    if (config.record) {
      const rect = readyInfo?.rect;
      if (!rect) throw new Error("recording needs the app window rect from the probe");
      await mkdir(dirname(config.record), { recursive: true });
      recording = startRecording(
        ffmpegPath(config.ffmpeg),
        config.record.replace(/\.mp4$/i, "") + ".raw.mp4",
        config.record,
        rect,
        config.seconds,
      );
      firstFrameWallMs = await Promise.race([
        recording.firstFrame,
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error("screen capture did not start")), 15_000),
        ),
      ]);
      if (debug) console.error(`recording: first frame at ${firstFrameWallMs}`);
    }
    const playerUrl = `${baseUrl}/player?start=${config.startSeconds}&seconds=${config.seconds}&trial=${trialId}`;
    const edgeArgs = [
      edgePath(),
      `--user-data-dir=${join(trialDir, "edge-profile")}`,
      "--autoplay-policy=no-user-gesture-required",
      "--no-first-run",
      "--no-default-browser-check",
      "--disable-extensions",
    ];
    if (config.record) edgeArgs.push("--window-position=0,0", "--window-size=1240,900");
    edgeArgs.push(`--app=${playerUrl}`);
    edge = Bun.spawn(edgeArgs, { stdout: "ignore", stderr: debug ? "pipe" : "ignore" });
    if (debug && edge) {
      void readLines(edge.stderr, (line) => console.error(`edge: ${line}`)).catch(() => {});
    }
    await Promise.race([
      stopped,
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("media playback timed out")), (config.seconds + 30) * 1000),
      ),
    ]);
    if (media.error) throw new Error(media.error);
    await new Promise((done) => setTimeout(done, 1500));
    if (recording) await stopRecording(recording);
  } finally {
    if (recording && recording.proc.exitCode === null) {
      await stopRecording(recording).catch(() => {});
    }
    await writeFile(stopFile, "stop").catch(() => {});
    await Promise.race([
      probe.exited,
      new Promise((done) => setTimeout(done, 5000)),
    ]).catch(() => {});
    await killTree(probe.pid).catch(() => {});
    await probeReader.catch(() => {});
    if (edge) await killTree(edge.pid).catch(() => {});
    await killTree(app.pid).catch(() => {});
    server.stop(true);
  }

  if (probeError) throw new Error(probeError);
  if (debug) {
    console.error(
      `observations=${observations.length} clock=${media.clock.length} ` +
        `firstObs=${observations[0]?.t ?? "n/a"} lastObs=${observations.at(-1)?.t ?? "n/a"} ` +
        `clockFirst=${media.clock[0]?.wallMs ?? "n/a"} clockLast=${media.clock.at(-1)?.wallMs ?? "n/a"}`,
    );
  }
  const probeErrorText = (await probeStderr).trim();
  if (probeErrorText) throw new Error(`ui probe: ${probeErrorText}`);

  const clock = new MediaClock(media.clock);
  const aligned: CaptionObservation[] = observations
    .map((observation) => {
      const mediaMs = clock.at(observation.t);
      return mediaMs === null
        ? null
        : { ...observation, mediaMs, tokens: tokenize(observation.text) };
    })
    .filter((value): value is CaptionObservation => value !== null)
    .filter(
      (observation) =>
        observation.mediaMs >= config.startSeconds * 1000 &&
        observation.mediaMs <= (config.startSeconds + config.seconds + 3) * 1000,
    )
    .sort((a, b) => a.mediaMs - b.mediaMs || a.t - b.t);

  const alignment = alignCaptions(cues, aligned);
  const targetCoverage =
    config.mode === "live"
      ? alignment.summary.partialCoverage
      : Math.max(alignment.summary.partialCoverage, alignment.summary.finalCoverage);
  let recordingMeta: TrialResult["recording"] = null;
  if (config.record && recording && media.start && firstFrameWallMs > 0) {
    const { trimMs } = await finalizeRecording(
      ffmpegPath(config.ffmpeg),
      recording,
      join(config.fixtureDir, manifest.media.file),
      config.startSeconds,
      config.seconds,
      firstFrameWallMs,
      media.start.wallMs,
    );
    recordingMeta = {
      file: recording.final,
      seconds: config.seconds,
      width: recording.width,
      height: recording.height,
      fps: 20,
      trimMs,
      mediaStartWallMs: media.start.wallMs,
      firstFrameWallMs,
      audioSource: "fixture track (what playback heard)",
    };
  }
  const result: TrialResult = {
    schemaVersion: 1,
    fixtureId: manifest.id,
    trial,
    app: {
      path: config.sourceUi ? "ui/src/main.tsx + ui/livelinguist-core.node" : config.app,
      sha256: await sha256(
        config.sourceUi ? join(ROOT, "ui", "livelinguist-core.node") : config.app,
      ),
    },
    commit:
      config.commit ??
      (config.sourceUi ? await commandOutput(["git", "rev-parse", "HEAD"]) : "installed-release"),
    config: {
      mode: config.mode,
      startSeconds: config.startSeconds,
      seconds: config.seconds,
      asrChunkMs: config.asrChunkMs ?? null,
      asrThreads: config.asrThreads ?? null,
      asrRule1Ms: config.asrRule1Ms ?? null,
      asrRule2Ms: config.asrRule2Ms ?? null,
      asrRule3Ms: config.asrRule3Ms ?? null,
      pollMs: config.pollMs,
      clickEveryN: config.clickEveryN,
    },
    media: {
      tickCount: media.clock.length,
      startWallMs: media.start?.wallMs ?? null,
      stopWallMs: media.stop?.wallMs ?? null,
      stopMediaMs: media.stop?.mediaMs ?? null,
      pollQuantumP95Ms: percentile(
        aligned.map((observation) => observation.scanMs + config.pollMs),
        95,
      ),
      error: media.error,
    },
    alignment: alignment.summary,
    cues: alignment.matches,
    click:
      config.clickAtMs > 0
        ? {
            requested: true,
            app: clickEvents.length,
            ok: clickEvents.length > 0 && clickEvents.every((click) => click.ok),
            maxGapMs: clickMaxGapMs(observations, clickEvents),
            mediaMs: clickEvents.length > 0 ? clock.at(clickEvents[0].t) : null,
          }
        : null,
    clickSeries:
      config.clickEveryN > 0
        ? summarizeClicks(clickEvents, simplifiedEvents, clock, observations)
        : null,
    recording: recordingMeta,
    valid: media.clock.length >= 2 && targetCoverage >= 0.8,
    contentFree: true,
  };
  assertContentFree(result);
  await writeFile(join(trialDir, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
  return result;
}

async function main(): Promise<void> {
  const config = await options();
  if (config.seconds <= 0 || config.repeat <= 0 || config.pollMs < 0) {
    throw new Error("seconds, repeat, and poll-ms must be positive");
  }
  const manifest = JSON.parse(await readFile(config.fixture, "utf8")) as FixtureManifest;
  await ensureFixture(manifest, config.fixtureDir);
  const srt = await readFile(join(config.fixtureDir, manifest.captions.file), "utf8");
  const cues = parseSrt(srt).filter(
    (cue) =>
      cue.endMs > config.startSeconds * 1000 &&
      cue.startMs < (config.startSeconds + config.seconds) * 1000,
  );
  if (cues.length < 5) throw new Error("fixture window has too few captions");
  const playerHtml = await readFile(PLAYER_PATH, "utf8");
  const vtt = srtToVtt(srt);
  await mkdir(config.out, { recursive: true });

  const results: TrialResult[] = [];
  for (let trial = 1; trial <= config.repeat; trial += 1) {
    const result = await runTrial(config, manifest, cues, playerHtml, vtt, trial);
    results.push(result);
    const clickText = result.clickSeries
      ? `; clicks ${result.clickSeries.clicked}/${result.clickSeries.requested} ` +
        `simplified=${result.clickSeries.simplified} p95=${result.clickSeries.p95Ms?.toFixed(0) ?? "n/a"} ms`
      : "";
    console.log(
      `trial ${trial}: partial p50=${result.alignment.partialP50Ms?.toFixed(0) ?? "n/a"} ms ` +
        `p95=${result.alignment.partialP95Ms?.toFixed(0) ?? "n/a"} ms ` +
        `coverage=${(result.alignment.partialCoverage * 100).toFixed(1)}%; ` +
        `final p50=${result.alignment.finalP50Ms?.toFixed(0) ?? "n/a"} ms ` +
        `p95=${result.alignment.finalP95Ms?.toFixed(0) ?? "n/a"} ms ` +
        `coverage=${(result.alignment.finalCoverage * 100).toFixed(1)}%${clickText}; ` +
        `valid=${result.valid}`,
    );
    if (result.recording) console.log(`recording: ${result.recording.file} (${result.recording.seconds}s)`);
  }

  const summary = {
    schemaVersion: 1,
    fixtureId: manifest.id,
    app: results[0]?.app,
    commit: results[0]?.commit,
    config: results[0]?.config,
    trials: results.map((result) => ({
      trial: result.trial,
      valid: result.valid,
      alignment: result.alignment,
      media: result.media,
      clickSeries: result.clickSeries,
      recording: result.recording,
    })),
    contentFree: true,
  };
  await writeFile(join(config.out, "summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
  console.log(`wrote ${join(config.out, "summary.json")}`);
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
