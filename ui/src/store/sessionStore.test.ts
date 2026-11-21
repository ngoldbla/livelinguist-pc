import { describe, expect, test } from "bun:test";
import type { Envelope } from "../bridge/generated/Envelope";
import { createSessionStore } from "./sessionStore";

function envelope(seq: number, event: Envelope["event"]): Envelope {
  return { v: 1, seq, ts: seq, event };
}

describe("session store", () => {
  test("applies events monotonically and drops stale seq", () => {
    const store = createSessionStore("fr-FR", "en");
    store.applyEnvelope(envelope(2, { type: "state", state: "listening", session_id: "s1" }));
    store.applyEnvelope(envelope(1, { type: "state", state: "setup", session_id: "old" }));
    expect(store.getSnapshot().sessionId).toBe("s1");
    expect(store.getSnapshot().lastSeq).toBe(2);
  });

  test("segments and simplified updates are reflected", () => {
    const store = createSessionStore("fr-FR", "en");
    store.applyEnvelope(
      envelope(3, {
        type: "segment",
        id: "a",
        text: "Bonjour tout le monde.",
        start_ms: 1000,
        end_ms: 2000,
      }),
    );
    store.applyEnvelope(
      envelope(4, {
        type: "simplified",
        id: "a",
        text: "Bonjour.",
        status: "clean",
        flags: [],
        latency_ms: 120,
      }),
    );
    const segment = store.getSnapshot().segments[0]!;
    expect(segment.text).toBe("Bonjour tout le monde.");
    expect(segment.simplified).toBe("Bonjour.");
    expect(segment.status).toBe("clean");
  });

  test("partials clear on an empty partial and overrun counts", () => {
    const store = createSessionStore("fr-FR", "en");
    store.applyEnvelope(
      envelope(5, {
        type: "partial",
        stream_id: "s",
        utterance_id: "u1",
        text: "Bonjour",
        confidence: null,
      }),
    );
    expect(store.getSnapshot().partial?.text).toBe("Bonjour");
    store.applyEnvelope(
      envelope(6, {
        type: "partial",
        stream_id: "s",
        utterance_id: "u1",
        text: "",
        confidence: null,
      }),
    );
    expect(store.getSnapshot().partial).toBeNull();
    store.applyEnvelope(
      envelope(7, {
        type: "audio_overrun",
        count: 2,
        dropped_ms: 500,
        recovery: "batch",
      }),
    );
    expect(store.getSnapshot().overrunCount).toBe(2);
  });

  test("snapshot replacement preserves the sequence floor and maps camelCase", () => {
    const store = createSessionStore("fr-FR", "en");
    store.applyEnvelope(envelope(9, { type: "state", state: "listening", session_id: "s1" }));
    store.replaceSnapshot({
      state: "summary",
      lastSeq: 20,
      sessionId: "s1",
      segments: [
        {
          id: "a",
          text: "Bonjour.",
          simplified: "Salut.",
          status: "clean",
          flags: [],
          startMs: 1000,
          endMs: 2000,
          detectedLanguage: "fr",
        },
      ],
      partial: null,
      reversionActive: false,
      overrunCount: 0,
      asrStatus: "simplify_only",
      language: "fr-FR",
    });
    const state = store.getSnapshot();
    expect(state.phase).toBe("summary");
    expect(state.lastSeq).toBe(20);
    expect(state.segments[0]?.detectedLanguage).toBe("fr");
    expect(state.segments[0]?.simplified).toBe("Salut.");
  });

  test("summary and error setters", () => {
    const store = createSessionStore("fr-FR", "en");
    store.setSummary({
      sessionId: "s1",
      language: "fr-FR",
      segments: 9,
      simplified: 9,
      durationS: 47.3,
      overrunCount: 0,
      lastSeq: 100,
    });
    expect(store.getSnapshot().summary?.segments).toBe(9);
    store.setError("ASR_MODEL_MISSING");
    expect(store.getSnapshot().lastError).toContain("ASR");
  });
});
