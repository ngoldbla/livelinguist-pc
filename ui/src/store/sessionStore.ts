import { useSyncExternalStore } from "react";
import type { Envelope } from "../bridge/generated/Envelope";
import type { RuntimeStatus } from "../bridge/generated/RuntimeStatus";
import type { Snapshot } from "../bridge/generated/Snapshot";

export type Phase = "setup" | "listening" | "paused" | "summary";
export type SimplifiedStatus = "clean" | "flagged" | "verbatim";
export type SimplificationMode = "on_demand" | "live";
export type AudioSource = "demo" | "mic" | "monitor" | "file" | "silence";

export interface Segment {
  id: string;
  text: string;
  simplified: string | null;
  status: SimplifiedStatus;
  flags: string[];
  startMs: number;
  endMs: number;
  detectedLanguage: string | null;
}

export interface PartialView {
  id: string;
  text: string;
}

export interface SessionSummary {
  sessionId: string;
  language: string;
  segments: number;
  simplified: number;
  durationS: number;
  overrunCount: number;
  lastSeq: number;
}

export interface SessionView {
  phase: Phase;
  language: string;
  uiLocale: string;
  source: AudioSource;
  mode: SimplificationMode;
  segments: Segment[];
  partial: PartialView | null;
  reversionActive: boolean;
  overrunCount: number;
  asrStatus: string;
  lastSeq: number;
  sessionId: string | null;
  runtime: RuntimeStatus | null;
  summary: SessionSummary | null;
  lastError: string | null;
}

export interface SessionStore {
  getSnapshot: () => SessionView;
  subscribe: (listener: () => void) => () => void;
  reset: () => void;
  applyEnvelope: (envelope: Envelope) => void;
  replaceSnapshot: (snapshot: Snapshot) => void;
  applySimplified: (result: {
    id: string;
    text: string | null;
    status: string;
    flags: string[];
  }) => void;
  setPhase: (phase: Phase) => void;
  setLanguage: (code: string, uiLocale: string) => void;
  setSource: (source: AudioSource) => void;
  setMode: (mode: SimplificationMode) => void;
  setAsrStatus: (status: string) => void;
  setRuntime: (runtime: RuntimeStatus) => void;
  setSummary: (summary: SessionSummary) => void;
  setError: (message: string) => void;
}

const initialState = (language = "fr-FR", uiLocale = "fr"): SessionView => ({
  phase: "setup",
  language,
  uiLocale,
  source: "demo",
  mode: "on_demand",
  segments: [],
  partial: null,
  reversionActive: false,
  overrunCount: 0,
  asrStatus: "simplify_only",
  lastSeq: 0,
  sessionId: null,
  runtime: null,
  summary: null,
  lastError: null,
});

export function createSessionStore(language = "fr-FR", uiLocale = "fr"): SessionStore {
  let state = initialState(language, uiLocale);
  const listeners = new Set<() => void>();

  const notify = () => {
    for (const listener of listeners) listener();
  };

  const set = (patch: Partial<SessionView>) => {
    state = { ...state, ...patch };
    notify();
  };

  const updateSegment = (id: string, patch: Partial<Segment>) => {
    state = {
      ...state,
      segments: state.segments.map((segment) =>
        segment.id === id ? { ...segment, ...patch } : segment,
      ),
    };
  };

  return {
    getSnapshot: () => state,
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    reset: () => {
      state = {
        ...initialState(state.language, state.uiLocale),
        source: state.source,
        mode: state.mode,
      };
      notify();
    },
    applyEnvelope: (envelope) => {
      if (envelope.seq <= state.lastSeq) return;
      const event = envelope.event;
      switch (event.type) {
        case "state":
          set({ phase: event.state, sessionId: event.session_id, lastSeq: envelope.seq });
          break;
        case "partial":
          set({
            partial: event.text ? { id: event.utterance_id, text: event.text } : null,
            lastSeq: envelope.seq,
          });
          break;
        case "segment":
          if (state.segments.some((segment) => segment.id === event.id)) {
            set({ lastSeq: envelope.seq });
            break;
          }
          set({
            segments: [
              ...state.segments,
              {
                id: event.id,
                text: event.text,
                simplified: null,
                status: "verbatim",
                flags: [],
                startMs: event.start_ms,
                endMs: event.end_ms,
                detectedLanguage: null,
              },
            ],
            lastSeq: envelope.seq,
          });
          break;
        case "simplified":
          updateSegment(event.id, {
            simplified: event.status === "verbatim" ? null : event.text,
            status: event.status,
            flags: event.flags,
          });
          set({ lastSeq: envelope.seq });
          break;
        case "reversion":
          set({ reversionActive: event.active, lastSeq: envelope.seq });
          break;
        case "audio_overrun":
          set({ overrunCount: event.count, lastSeq: envelope.seq });
          break;
        case "runtime_status":
          set({ runtime: event.status, lastSeq: envelope.seq });
          break;
        case "error":
          set({ lastError: `${event.code}: ${event.message}`, lastSeq: envelope.seq });
          break;
        default:
          set({ lastSeq: envelope.seq });
          break;
      }
    },
    replaceSnapshot: (snapshot) => {
      set({
        phase: snapshot.state,
        sessionId: snapshot.sessionId,
        language: snapshot.language,
        lastSeq: Math.max(state.lastSeq, snapshot.lastSeq),
        partial: snapshot.partial
          ? { id: snapshot.partial.utteranceId, text: snapshot.partial.text }
          : null,
        reversionActive: snapshot.reversionActive,
        overrunCount: snapshot.overrunCount,
        asrStatus: snapshot.asrStatus,
        segments: snapshot.segments.map((segment) => ({
          id: segment.id,
          text: segment.text,
          simplified: segment.simplified,
          status: segment.status,
          flags: segment.flags,
          startMs: segment.startMs,
          endMs: segment.endMs,
          detectedLanguage: segment.detectedLanguage,
        })),
      });
    },
    applySimplified: (result) => {
      updateSegment(result.id, {
        simplified: result.text,
        status:
          result.status === "clean"
            ? "clean"
            : result.status === "flagged"
              ? "flagged"
              : "verbatim",
        flags: result.flags,
      });
      notify();
    },
    setPhase: (phase) => set({ phase }),
    setLanguage: (code, uiLocale) => set({ language: code, uiLocale }),
    setSource: (source) => set({ source }),
    setMode: (mode) => set({ mode }),
    setAsrStatus: (asrStatus) => set({ asrStatus }),
    setRuntime: (runtime) => set({ runtime }),
    setSummary: (summary) => set({ summary, phase: "summary" }),
    setError: (lastError) => set({ lastError }),
  };
}

export function useSession(store: SessionStore): SessionView {
  return useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);
}
