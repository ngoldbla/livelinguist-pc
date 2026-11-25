import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

const ROOT = resolve(import.meta.dir, "..");

interface Candidate {
  id: string;
  label: string;
  sourceUi: boolean;
  mode: "live" | "overlay";
  asrChunkMs?: number;
  asrThreads?: number;
  asrRule1Ms?: number;
  asrRule2Ms?: number;
  asrRule3Ms?: number;
}

interface TrialSummary {
  valid: boolean;
  alignment: {
    partialCoverage: number;
    finalCoverage: number;
    partialP50Ms: number | null;
    partialP95Ms: number | null;
    finalP50Ms: number | null;
    finalP95Ms: number | null;
    meanPartialScore: number | null;
    meanFinalScore: number | null;
  };
}

interface RunSummary {
  config: Record<string, unknown>;
  trials: TrialSummary[];
}

function argument(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 ? (process.argv[index + 1] ?? fallback) : fallback;
}

function installedApp(): string {
  return join(
    process.env.LOCALAPPDATA ?? "",
    "Programs",
    "LiveLinguist",
    "livelinguist.exe",
  );
}

const seconds = argument("--seconds", "60");
const repeat = argument("--repeat", "1");
const from = argument("--from");
const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const out = resolve(argument("--out", join(ROOT, "target", "autoresearch", "optimization", stamp)));
await mkdir(out, { recursive: true });

const candidates: Candidate[] = [
  {
    id: "release-live",
    label: "Installed RC, live view (baseline)",
    sourceUi: false,
    mode: "live",
  },
  {
    id: "release-overlay",
    label: "Installed RC, overlay (baseline)",
    sourceUi: false,
    mode: "overlay",
  },
  {
    id: "source-overlay-1100",
    label: "Overlay partial + original 1100 ms chunk",
    sourceUi: true,
    mode: "overlay",
    asrChunkMs: 1100,
    asrThreads: 2,
    asrRule1Ms: 2400,
    asrRule2Ms: 1200,
    asrRule3Ms: 20_000,
  },
  {
    id: "source-overlay-560",
    label: "Overlay partial + 560 ms input chunk",
    sourceUi: true,
    mode: "overlay",
    asrChunkMs: 560,
    asrThreads: 2,
    asrRule1Ms: 2400,
    asrRule2Ms: 1200,
    asrRule3Ms: 20_000,
  },
  {
    id: "source-overlay-80",
    label: "Overlay partial + 80 ms input chunk",
    sourceUi: true,
    mode: "overlay",
    asrChunkMs: 80,
    asrThreads: 2,
    asrRule1Ms: 2400,
    asrRule2Ms: 1200,
    asrRule3Ms: 20_000,
  },
  {
    id: "source-overlay-fast-endpoint",
    label: "Overlay partial + 80 ms + fast endpoint + 4 threads",
    sourceUi: true,
    mode: "overlay",
    asrChunkMs: 80,
    asrThreads: 4,
    asrRule1Ms: 1200,
    asrRule2Ms: 400,
    asrRule3Ms: 10_000,
  },
];

const rows: Array<Record<string, unknown>> = [];
for (const candidate of candidates) {
  const candidateOut = join(from ? resolve(from) : out, candidate.id);
  if (!from) {
    await mkdir(candidateOut, { recursive: true });
    await writeFile(
      join(candidateOut, "hypothesis.md"),
      `# ${candidate.id}\n\n${candidate.label}\n\n` +
        `Expected mechanism: reduce buffering/endpoint/presentation delay without losing cue coverage or text similarity.\n`,
    );
  }
  if (from) {
    console.log(`\n=== ${candidate.id}: ${candidate.label} (reuse) ===`);
  } else {
    const args = [
      "autoresearch/benchmark.ts",
      "--mode",
      candidate.mode,
      "--seconds",
      seconds,
      "--repeat",
      repeat,
      "--out",
      candidateOut,
    ];
    if (candidate.sourceUi) {
      args.push("--source-ui");
    } else {
      args.push("--app", installedApp());
    }
    for (const [flag, value] of [
      ["--asr-chunk-ms", candidate.asrChunkMs],
      ["--asr-threads", candidate.asrThreads],
      ["--asr-rule1-ms", candidate.asrRule1Ms],
      ["--asr-rule2-ms", candidate.asrRule2Ms],
      ["--asr-rule3-ms", candidate.asrRule3Ms],
    ] as const) {
      if (value !== undefined) args.push(flag, String(value));
    }
    console.log(`\n=== ${candidate.id}: ${candidate.label} ===`);
    const proc = Bun.spawn(["bun", ...args], { cwd: ROOT, stdout: "inherit", stderr: "inherit" });
    const exitCode = await proc.exited;
    if (exitCode !== 0) {
      rows.push({ id: candidate.id, label: candidate.label, error: "benchmark failed" });
      continue;
    }
  }
  const summary = JSON.parse(await readFile(join(candidateOut, "summary.json"), "utf8")) as RunSummary;
  const validTrials = summary.trials.filter((trial) => trial.valid);
  const mean = (values: Array<number | null>) => {
    const present = values.filter((value): value is number => value !== null);
    return present.length
      ? present.reduce((sum, value) => sum + value, 0) / present.length
      : null;
  };
  const partialCoverage = mean(summary.trials.map((trial) => trial.alignment.partialCoverage));
  const finalCoverage = mean(summary.trials.map((trial) => trial.alignment.finalCoverage));
  const partialP95 = mean(summary.trials.map((trial) => trial.alignment.partialP95Ms));
  const finalP95 = mean(summary.trials.map((trial) => trial.alignment.finalP95Ms));
  const partialP50 = mean(summary.trials.map((trial) => trial.alignment.partialP50Ms));
  const finalP50 = mean(summary.trials.map((trial) => trial.alignment.finalP50Ms));
  const partialScore = mean(summary.trials.map((trial) => trial.alignment.meanPartialScore));
  const finalScore = mean(summary.trials.map((trial) => trial.alignment.meanFinalScore));
  const usePartial =
    summary.config.mode === "live" || (partialCoverage ?? 0) > 0;
  const metric = usePartial ? partialP95 : finalP95;
  const coverage = usePartial ? partialCoverage : finalCoverage;
  const score = usePartial ? partialScore : finalScore;
  rows.push({
    id: candidate.id,
    label: candidate.label,
    config: summary.config,
    validTrials: validTrials.length,
    totalTrials: summary.trials.length,
    tier: usePartial ? "partial" : "final",
    metricP95Ms: metric,
    metricP50Ms: usePartial ? partialP50 : finalP50,
    coverage,
    meanScore: score,
    eligible:
      validTrials.length === summary.trials.length &&
      (coverage ?? 0) >= 0.8 &&
      (score ?? 0) >= 0.85,
  });
}

const baseline = rows.find((row) => row.id === "release-overlay");
const control = rows.find((row) => row.id === "source-overlay-1100");
const baselineMetric = typeof baseline?.metricP95Ms === "number" ? baseline.metricP95Ms : null;
const controlMetric = typeof control?.metricP95Ms === "number" ? control.metricP95Ms : null;
for (const row of rows) {
  const metric = row.metricP95Ms;
  row.improvementVsReleaseOverlay =
    typeof metric === "number" && baselineMetric !== null
      ? (baselineMetric - metric) / baselineMetric
      : null;
  row.improvementVsOverlayControl =
    typeof metric === "number" && controlMetric !== null
      ? (controlMetric - metric) / controlMetric
      : null;
}
const eligible = rows
  .filter((row) => row.eligible === true && typeof row.metricP95Ms === "number")
  .sort((a, b) => Number(a.metricP95Ms) - Number(b.metricP95Ms));
const journal = {
  schemaVersion: 1,
  createdAt: new Date().toISOString(),
  seconds: Number(seconds),
  repeat: Number(repeat),
  reusedFrom: from ? resolve(from) : null,
  objective: "minimize caption-cue-start to visible caption p95",
  constraints: ["coverage >= 0.80", "mean token similarity >= 0.85", "content-free outputs"],
  rows,
  best: eligible[0] ?? null,
  contentFree: true,
};
await writeFile(join(out, "journal.json"), `${JSON.stringify(journal, null, 2)}\n`);
console.log(`\nbest: ${eligible[0]?.id ?? "none eligible"}`);
console.log(`wrote ${join(out, "journal.json")}`);
