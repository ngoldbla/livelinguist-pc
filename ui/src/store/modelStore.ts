// First-run model download state (T16). Download envelopes come from their
// own emitter, so this store keeps its own `seq` and resets it when a new
// download starts.
import { useSyncExternalStore } from "react";
import type { Envelope } from "../bridge/generated/Envelope";

export interface ModelStatus {
  id: string;
  title: string;
  bytes: number;
  present: boolean;
  verified: boolean;
}

export type ModelPhase = "missing" | "downloading" | "ready" | "error";

export interface ModelEntry extends ModelStatus {
  phase: ModelPhase;
  /** Bytes on disk per file, from progress events. */
  files: Record<string, number>;
  doneBytes: number;
}

export interface ModelError {
  id: string;
  code: string;
  reason: string;
}

export interface ModelView {
  models: ModelEntry[];
  downloading: boolean;
  error: ModelError | null;
  lastSeq: number;
}

export interface ModelStore {
  getSnapshot: () => ModelView;
  subscribe: (listener: () => void) => () => void;
  setStatus: (status: ModelStatus[]) => void;
  /** A new download run: clears the error and resets `seq`. */
  begin: () => void;
  /** The download could not start (for example demo mode). */
  fail: (error: ModelError) => void;
  applyEnvelope: (envelope: Envelope) => void;
}

export const allReady = (view: ModelView): boolean =>
  view.models.length > 0 && view.models.every((model) => model.phase === "ready");

export const percent = (model: ModelEntry): number =>
  model.bytes > 0 ? Math.min(100, Math.floor((model.doneBytes / model.bytes) * 100)) : 0;

export const megabytes = (bytes: number): string => `${Math.round(bytes / 1_000_000)} MB`;

function entry(status: ModelStatus): ModelEntry {
  return {
    ...status,
    phase: status.verified ? "ready" : "missing",
    files: {},
    doneBytes: status.verified ? status.bytes : 0,
  };
}

export function createModelStore(): ModelStore {
  let state: ModelView = { models: [], downloading: false, error: null, lastSeq: 0 };
  const listeners = new Set<() => void>();
  const set = (patch: Partial<ModelView>) => {
    state = { ...state, ...patch };
    for (const listener of listeners) listener();
  };
  const update = (id: string, change: (model: ModelEntry) => ModelEntry) =>
    state.models.map((model) => (model.id === id ? change(model) : model));

  return {
    getSnapshot: () => state,
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    setStatus: (status) => {
      const previous = new Map(state.models.map((model) => [model.id, model]));
      set({
        models: status.map((item) => {
          const fresh = entry(item);
          const old = previous.get(item.id);
          return old && fresh.phase !== "ready"
            ? { ...fresh, files: old.files, doneBytes: old.doneBytes }
            : fresh;
        }),
      });
    },
    begin: () => {
      set({
        downloading: true,
        error: null,
        lastSeq: 0,
        models: state.models.map((model) =>
          model.phase === "ready" ? model : { ...model, phase: "downloading" },
        ),
      });
    },
    fail: (error) => {
      set({
        downloading: false,
        error,
        models: update(error.id, (model) => ({ ...model, phase: "error" })),
      });
    },
    applyEnvelope: (envelope) => {
      if (envelope.seq <= state.lastSeq) return;
      const event = envelope.event;
      switch (event.type) {
        case "model_progress": {
          const models = update(event.id, (model) => {
            const files = { ...model.files, [event.file]: event.done_bytes };
            const doneBytes = Object.values(files).reduce((sum, value) => sum + value, 0);
            return { ...model, phase: "downloading", files, doneBytes };
          });
          set({ models, lastSeq: envelope.seq });
          break;
        }
        case "model_ready": {
          const models = update(event.id, (model) => ({
            ...model,
            phase: "ready",
            present: true,
            verified: true,
            doneBytes: model.bytes,
          }));
          const done = models.every((model) => model.phase === "ready");
          set({ models, lastSeq: envelope.seq, downloading: !done });
          break;
        }
        case "model_error":
          set({
            models: update(event.id, (model) => ({ ...model, phase: "error" })),
            error: { id: event.id, code: event.code, reason: event.reason },
            downloading: false,
            lastSeq: envelope.seq,
          });
          break;
        default:
          break;
      }
    },
  };
}

export function useModels(store: ModelStore): ModelView {
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);
}
