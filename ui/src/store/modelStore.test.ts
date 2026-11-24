import { describe, expect, test } from "bun:test";
import type { Envelope } from "../bridge/generated/Envelope";
import { allReady, createModelStore, percent } from "./modelStore";

const status = [
  { id: "asr", title: "Speech recognition", bytes: 1000, present: false, verified: false },
  { id: "llm", title: "Simplifier", bytes: 400, present: false, verified: false },
];

function envelope(seq: number, event: Envelope["event"]): Envelope {
  return { v: 1, seq, ts: seq, event };
}

const progress = (seq: number, id: string, file: string, done: number, total: number) =>
  envelope(seq, {
    type: "model_progress",
    id,
    file,
    done_bytes: done,
    total_bytes: total,
  });

describe("model store", () => {
  test("status sets missing vs ready", () => {
    const store = createModelStore();
    store.setStatus([status[0]!, { ...status[1]!, present: true, verified: true }]);
    const view = store.getSnapshot();
    expect(view.models[0]?.phase).toBe("missing");
    expect(view.models[1]?.phase).toBe("ready");
    expect(allReady(view)).toBe(false);
  });

  test("progress then ready", () => {
    const store = createModelStore();
    store.setStatus(status);
    store.begin();
    store.applyEnvelope(progress(1, "asr", "encoder", 300, 900));
    store.applyEnvelope(progress(2, "asr", "tokens", 100, 100));
    expect(store.getSnapshot().models[0]?.doneBytes).toBe(400);
    expect(percent(store.getSnapshot().models[0]!)).toBe(40);
    store.applyEnvelope(envelope(3, { type: "model_ready", id: "asr" }));
    expect(store.getSnapshot().downloading).toBe(true);
    store.applyEnvelope(envelope(4, { type: "model_ready", id: "llm" }));
    const view = store.getSnapshot();
    expect(allReady(view)).toBe(true);
    expect(view.downloading).toBe(false);
    expect(percent(view.models[1]!)).toBe(100);
  });

  test("progress then error then retry resets seq and keeps bytes", () => {
    const store = createModelStore();
    store.setStatus(status);
    store.begin();
    store.applyEnvelope(progress(1, "asr", "encoder", 500, 900));
    store.applyEnvelope(
      envelope(2, {
        type: "model_error",
        id: "asr",
        code: "CANCELLED",
        reason: "download cancelled",
      }),
    );
    let view = store.getSnapshot();
    expect(view.error?.code).toBe("CANCELLED");
    expect(view.downloading).toBe(false);
    expect(view.models[0]?.phase).toBe("error");
    // Retry: a new download run starts its own seq at 1 again.
    store.setStatus([{ ...status[0]!, present: true }, status[1]!]);
    store.begin();
    view = store.getSnapshot();
    expect(view.error).toBeNull();
    expect(view.models[0]?.phase).toBe("downloading");
    expect(view.models[0]?.doneBytes).toBe(500);
    store.applyEnvelope(progress(1, "asr", "encoder", 900, 900));
    expect(store.getSnapshot().models[0]?.doneBytes).toBe(900);
  });

  test("ignores envelopes at or below the last seq", () => {
    const store = createModelStore();
    store.setStatus(status);
    store.begin();
    store.applyEnvelope(progress(5, "asr", "encoder", 600, 900));
    store.applyEnvelope(progress(4, "asr", "encoder", 100, 900));
    store.applyEnvelope(progress(5, "asr", "encoder", 200, 900));
    expect(store.getSnapshot().models[0]?.doneBytes).toBe(600);
    expect(store.getSnapshot().lastSeq).toBe(5);
  });

  test("fail marks the model and stops downloading", () => {
    const store = createModelStore();
    store.setStatus(status);
    store.begin();
    store.fail({ id: "asr", code: "DEMO", reason: "not available in demo mode" });
    expect(store.getSnapshot().downloading).toBe(false);
    expect(store.getSnapshot().error?.reason).toBe("not available in demo mode");
  });
});
