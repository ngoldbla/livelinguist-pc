import { describe, expect, test } from "bun:test";
import {
  alignCaptions,
  MediaClock,
  parseSrt,
  percentile,
  srtToVtt,
  tokenize,
  type CaptionObservation,
} from "./align";

const srt = `1
00:00:00,700 --> 00:00:04,310
Hello and welcome to Programming for
Everybody.

2
00:00:04,310 --> 00:00:05,910
My name is Charles.
`;

describe("caption alignment", () => {
  test("parses SRT and converts commas to VTT dots", () => {
    const cues = parseSrt(srt);
    expect(cues).toHaveLength(2);
    expect(cues[0]?.startMs).toBe(700);
    expect(cues[0]?.tokens.slice(0, 3)).toEqual(["hello", "and", "welcome"]);
    expect(srtToVtt(srt)).toContain("00:00:00.700 --> 00:00:04.310");
  });

  test("maps wall time through a stalling media clock", () => {
    const clock = new MediaClock([
      { wallMs: 1000, mediaMs: 0 },
      { wallMs: 2000, mediaMs: 500 },
      { wallMs: 4000, mediaMs: 500 },
      { wallMs: 5000, mediaMs: 1500 },
    ]);
    expect(clock.at(1500)).toBe(250);
    expect(clock.at(3000)).toBe(500);
    expect(clock.at(4500)).toBe(1000);
  });

  test("finds partial and final cue onset without storing text in the summary", () => {
    const cues = parseSrt(srt);
    const observations: CaptionObservation[] = [
      {
        t: 0,
        kind: "partial",
        text: "Hello and welcome to programming for everybody",
        scanMs: 10,
        mediaMs: 1450,
        tokens: tokenize("Hello and welcome to programming for everybody"),
      },
      {
        t: 0,
        kind: "final",
        text: "Hello and welcome to programming for everybody. My name is Charles.",
        scanMs: 10,
        mediaMs: 5900,
        tokens: tokenize("Hello and welcome to programming for everybody. My name is Charles."),
      },
    ];
    const { matches, summary } = alignCaptions(cues, observations);
    expect(matches[0]?.partialDelayMs).toBe(750);
    expect(matches[1]?.partialDelayMs).toBeNull();
    expect(matches[0]?.finalDelayMs).toBe(5200);
    expect(matches[1]?.finalDelayMs).toBe(1590);
    expect(summary.partialCoverage).toBe(0.5);
    expect(summary.finalCoverage).toBe(1);
    expect(JSON.stringify(summary)).not.toContain("hello");
  });

  test("falls back to the full cue when the opening words are mistranscribed", () => {
    const cue = {
      index: 1,
      startMs: 1000,
      endMs: 3000,
      text: "one two three four five",
      tokens: tokenize("one two three four five"),
    };
    const observation: CaptionObservation = {
      t: 0,
      kind: "partial",
      text: "one too three four five",
      scanMs: 10,
      mediaMs: 2200,
      tokens: tokenize("one too three four five"),
    };
    const { summary } = alignCaptions([cue], [observation]);
    expect(summary.partialMatched).toBe(1);
    expect(summary.partialP50Ms).toBe(1200);
  });

  test("uses nearest-rank percentiles", () => {
    expect(percentile([4, 1, 3, 2], 50)).toBe(2);
    expect(percentile([4, 1, 3, 2], 95)).toBe(4);
    expect(percentile([], 95)).toBeNull();
  });
});
