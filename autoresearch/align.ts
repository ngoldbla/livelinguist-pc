export interface CaptionCue {
  index: number;
  startMs: number;
  endMs: number;
  text: string;
  tokens: string[];
}

export interface CaptionObservation {
  t: number;
  kind: "partial" | "final";
  text: string;
  scanMs: number;
  mediaMs: number;
  tokens: string[];
}

export interface MediaClockSample {
  wallMs: number;
  mediaMs: number;
}

export interface CueMatch {
  cue: number;
  startMs: number;
  endMs: number;
  partialDelayMs: number | null;
  finalDelayMs: number | null;
  partialScore: number | null;
  finalScore: number | null;
}

export interface AlignmentSummary {
  cueCount: number;
  partialMatched: number;
  finalMatched: number;
  partialCoverage: number;
  finalCoverage: number;
  partialP50Ms: number | null;
  partialP95Ms: number | null;
  finalP50Ms: number | null;
  finalP95Ms: number | null;
  firstPartialMediaMs: number | null;
  firstFinalMediaMs: number | null;
  meanPartialScore: number | null;
  meanFinalScore: number | null;
}

function timestampMs(value: string): number {
  const match = /^(\d+):(\d{2}):(\d{2})[,.](\d{1,3})$/.exec(value.trim());
  if (!match) throw new Error(`invalid SRT timestamp: ${value}`);
  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  const seconds = Number(match[3]);
  const millis = Number(match[4].padEnd(3, "0"));
  return ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis;
}

export function normalizeText(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[’‘`]/g, "'")
    .replace(/[^\p{L}\p{N}'\s]/gu, " ")
    .replace(/\s+/g, " ")
    .trim();
}

export function tokenize(value: string): string[] {
  const normalized = normalizeText(value);
  return normalized ? normalized.split(" ") : [];
}

export function parseSrt(value: string): CaptionCue[] {
  const cues: CaptionCue[] = [];
  for (const block of value.replace(/\r/g, "").split(/\n{2,}/)) {
    const lines = block.split("\n").map((line) => line.trim());
    if (lines.length < 3) continue;
    const index = Number(lines[0]);
    const timing = lines[1];
    if (!Number.isFinite(index) || !timing.includes("-->")) continue;
    const [start, end] = timing.split("-->").map((part) => part.trim());
    const text = lines.slice(2).join(" ");
    cues.push({
      index,
      startMs: timestampMs(start),
      endMs: timestampMs(end),
      text,
      tokens: tokenize(text),
    });
  }
  return cues.sort((a, b) => a.startMs - b.startMs || a.index - b.index);
}

export function srtToVtt(value: string): string {
  return `WEBVTT\n\n${value.replace(/\r/g, "").replace(/(\d{2}:\d{2}:\d{2}),(\d{3})/g, "$1.$2")}\n`;
}

export class MediaClock {
  private readonly samples: MediaClockSample[];

  constructor(samples: MediaClockSample[]) {
    this.samples = [...samples].sort((a, b) => a.wallMs - b.wallMs);
  }

  at(wallMs: number): number | null {
    if (this.samples.length === 0) return null;
    const first = this.samples[0]!;
    if (wallMs <= first.wallMs) return first.mediaMs + (wallMs - first.wallMs);
    for (let index = 1; index < this.samples.length; index += 1) {
      const right = this.samples[index]!;
      const left = this.samples[index - 1]!;
      if (wallMs <= right.wallMs) {
        const span = right.wallMs - left.wallMs;
        if (span <= 0) return right.mediaMs;
        const fraction = (wallMs - left.wallMs) / span;
        return left.mediaMs + (right.mediaMs - left.mediaMs) * fraction;
      }
    }
    const last = this.samples[this.samples.length - 1]!;
    return last.mediaMs + (wallMs - last.wallMs);
  }
}

function tokenDistance(left: string[], right: string[]): number {
  if (left.length === 0) return right.length;
  if (right.length === 0) return left.length;
  let previous = Array.from({ length: right.length + 1 }, (_, index) => index);
  for (let i = 1; i <= left.length; i += 1) {
    const current = new Array<number>(right.length + 1);
    current[0] = i;
    for (let j = 1; j <= right.length; j += 1) {
      const cost = left[i - 1] === right[j - 1] ? 0 : 1;
      current[j] = Math.min(
        current[j - 1]! + 1,
        previous[j]! + 1,
        previous[j - 1]! + cost,
      );
    }
    previous = current;
  }
  return previous[right.length]!;
}

function similarity(left: string[], right: string[]): number {
  const denominator = Math.max(left.length, right.length);
  if (denominator === 0) return 0;
  return 1 - tokenDistance(left, right) / denominator;
}

function bestWindow(hypothesis: string[], reference: string[]): number {
  if (hypothesis.length === 0 || reference.length === 0) return 0;
  if (reference.length > hypothesis.length) {
    return similarity(hypothesis, reference.slice(0, hypothesis.length));
  }
  let best = 0;
  for (let start = 0; start + reference.length <= hypothesis.length; start += 1) {
    best = Math.max(best, similarity(hypothesis.slice(start, start + reference.length), reference));
  }
  return best;
}

export function percentile(values: number[], p: number): number | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.max(1, Math.ceil((p / 100) * sorted.length));
  return sorted[Math.min(rank, sorted.length) - 1]!;
}

export function mean(values: number[]): number | null {
  if (values.length === 0) return null;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

export function alignCaptions(
  cues: CaptionCue[],
  observations: CaptionObservation[],
  threshold = 0.75,
): { matches: CueMatch[]; summary: AlignmentSummary } {
  const matches: CueMatch[] = [];
  let lastPartialTime = Number.NEGATIVE_INFINITY;
  let lastFinalTime = Number.NEGATIVE_INFINITY;

  for (const cue of cues) {
    const anchor = cue.tokens.slice(0, Math.min(3, cue.tokens.length));
    let partialMatch = findMatch(
      observations,
      "partial",
      cue,
      anchor,
      lastPartialTime,
      threshold,
    );
    if (partialMatch === null && cue.tokens.length > anchor.length) {
      partialMatch = findMatch(
        observations,
        "partial",
        cue,
        cue.tokens,
        lastPartialTime,
        threshold,
      );
    }
    const finalMatch = findMatch(
      observations,
      "final",
      cue,
      cue.tokens,
      lastFinalTime,
      threshold,
    );
    if (partialMatch) lastPartialTime = partialMatch.observation.mediaMs;
    if (finalMatch) lastFinalTime = finalMatch.observation.mediaMs;
    matches.push({
      cue: cue.index,
      startMs: cue.startMs,
      endMs: cue.endMs,
      partialDelayMs: partialMatch ? partialMatch.observation.mediaMs - cue.startMs : null,
      finalDelayMs: finalMatch ? finalMatch.observation.mediaMs - cue.startMs : null,
      partialScore: partialMatch?.score ?? null,
      finalScore: finalMatch?.score ?? null,
    });
  }

  const partialDelays = matches
    .map((match) => match.partialDelayMs)
    .filter((value): value is number => value !== null);
  const finalDelays = matches
    .map((match) => match.finalDelayMs)
    .filter((value): value is number => value !== null);
  const partialScores = matches
    .map((match) => match.partialScore)
    .filter((value): value is number => value !== null);
  const finalScores = matches
    .map((match) => match.finalScore)
    .filter((value): value is number => value !== null);
  const partialObservations = observations.filter((item) => item.kind === "partial");
  const finalObservations = observations.filter((item) => item.kind === "final");

  return {
    matches,
    summary: {
      cueCount: cues.length,
      partialMatched: partialDelays.length,
      finalMatched: finalDelays.length,
      partialCoverage: cues.length ? partialDelays.length / cues.length : 0,
      finalCoverage: cues.length ? finalDelays.length / cues.length : 0,
      partialP50Ms: percentile(partialDelays, 50),
      partialP95Ms: percentile(partialDelays, 95),
      finalP50Ms: percentile(finalDelays, 50),
      finalP95Ms: percentile(finalDelays, 95),
      firstPartialMediaMs: partialObservations[0]?.mediaMs ?? null,
      firstFinalMediaMs: finalObservations[0]?.mediaMs ?? null,
      meanPartialScore: mean(partialScores),
      meanFinalScore: mean(finalScores),
    },
  };
}

function findMatch(
  observations: CaptionObservation[],
  kind: "partial" | "final",
  cue: CaptionCue,
  reference: string[],
  lastTime: number,
  threshold: number,
): { observation: CaptionObservation; score: number } | null {
  let best: { observation: CaptionObservation; score: number } | null = null;
  for (const observation of observations) {
    if (observation.kind !== kind) continue;
    if (observation.mediaMs < cue.startMs - 500 || observation.mediaMs > cue.endMs + 10_000) {
      continue;
    }
    if (observation.mediaMs + 50 < lastTime) continue;
    const score = bestWindow(observation.tokens, reference);
    if (score < threshold) continue;
    if (
      best === null ||
      score > best.score + 0.0001 ||
      (Math.abs(score - best.score) <= 0.0001 && observation.mediaMs < best.observation.mediaMs)
    ) {
      best = { observation, score };
    }
  }
  return best;
}
