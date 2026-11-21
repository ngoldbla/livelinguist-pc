import { createRequire } from "node:module";
import type {
  AudioSource,
  SessionSummary,
  SimplificationMode,
  SimplifiedStatus,
} from "../store/sessionStore";
import type { Envelope } from "./generated/Envelope";
import type { RuntimeStatus } from "./generated/RuntimeStatus";
import type { Snapshot } from "./generated/Snapshot";

export interface StartOptions {
  language: string;
  source: AudioSource;
  mode: SimplificationMode;
  detection: boolean;
  scenario?: string;
}

export interface SimplifyResult {
  id: string;
  text: string | null;
  status: SimplifiedStatus;
  flags: string[];
}

export interface WindowPolicy {
  applied: boolean;
  backend: string;
  notes: string[];
}

export interface AudioSourceInfo {
  id: string;
  label: string;
  kind: string;
  available: boolean;
  note?: string;
}

export interface BackendProbe {
  backend: string;
  devices: Array<{ id: string; kind: string; available: boolean }>;
  mockCalibration: boolean;
}

export interface Transport {
  readonly kind: "native" | "demo";
  start(options: StartOptions): Promise<void>;
  feedDemo(): Promise<void>;
  stop(): Promise<SessionSummary>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  simplify(id: string): Promise<SimplifyResult>;
  simplifyText(text: string): Promise<SimplifyResult>;
  snapshot(): Promise<Snapshot>;
  runtimeStatus(): Promise<RuntimeStatus>;
  setWindowMode(mode: "console" | "overlay" | "expanded"): Promise<WindowPolicy>;
  exportTranscript(format: "markdown" | "plain" | "csv", path: string): Promise<string>;
  listSources(): Promise<AudioSourceInfo[]>;
  applyRuntimeConfig(layer: unknown): Promise<RuntimeStatus>;
  applyScenario(scenario: string): Promise<RuntimeStatus>;
  probeBackends(): Promise<BackendProbe>;
  calibrate(): Promise<RuntimeStatus>;
}

interface NativeClient {
  start(requestJson: string, onEvent: (error: Error | null, value: string) => void): void;
  feedDemo(): void;
  stop(): string;
  pause(): void;
  resume(): void;
  simplify(id: string): string;
  simplifyText(text: string): string;
  snapshot(): string;
  runtimeStatus(): string;
  setWindowMode(mode: string, nonce: string): string;
  export(format: string, path: string): string;
  listAudioSources(): string;
  applyRuntimeConfig(layerJson: string): string;
  applyScenario(scenario: string): string;
  probeBackends(): string;
  calibrate(): string;
}

interface NativeAddon {
  CoreClient: new () => NativeClient;
}

export const windowNonce = String(process.pid);
export const windowTitle = `Live Linguist — ${windowNonce}`;

let nativeAddon: NativeAddon | null | undefined;

function loadNative(): NativeAddon | null {
  if (nativeAddon !== undefined) return nativeAddon;
  try {
    const require = createRequire(import.meta.url);
    nativeAddon = require("../../livelinguist-core.node") as NativeAddon;
  } catch {
    nativeAddon = null;
  }
  return nativeAddon;
}

export function createTransport(onEvent: (envelope: Envelope) => void): Transport {
  const addon = loadNative();
  if (addon) return new NativeTransport(addon, onEvent);
  return new DemoTransport(onEvent);
}

class NativeTransport implements Transport {
  readonly kind = "native" as const;
  private client: NativeClient;

  constructor(addon: NativeAddon, onEvent: (envelope: Envelope) => void) {
    this.client = new addon.CoreClient();
    this.onEvent = onEvent;
  }

  private onEvent: (envelope: Envelope) => void;

  async start(options: StartOptions): Promise<void> {
    this.client.start(
      JSON.stringify({
        language: options.language,
        source: options.source,
        mode: options.mode,
        detection: options.detection,
        scenario: options.scenario ?? "mock",
      }),
      (error, value) => {
        if (error) return;
        try {
          this.onEvent(JSON.parse(value) as Envelope);
        } catch {
          // Ignore malformed bridge payloads; the sequence gate stays intact.
        }
      },
    );
  }

  async feedDemo(): Promise<void> {
    this.client.feedDemo();
  }

  async stop(): Promise<SessionSummary> {
    return JSON.parse(this.client.stop()) as SessionSummary;
  }

  async pause(): Promise<void> {
    this.client.pause();
  }

  async resume(): Promise<void> {
    this.client.resume();
  }

  async simplify(id: string): Promise<SimplifyResult> {
    return JSON.parse(this.client.simplify(id)) as SimplifyResult;
  }

  async simplifyText(text: string): Promise<SimplifyResult> {
    return JSON.parse(this.client.simplifyText(text)) as SimplifyResult;
  }

  async snapshot(): Promise<Snapshot> {
    return JSON.parse(this.client.snapshot()) as Snapshot;
  }

  async runtimeStatus(): Promise<RuntimeStatus> {
    return JSON.parse(this.client.runtimeStatus()) as RuntimeStatus;
  }

  async setWindowMode(mode: "console" | "overlay" | "expanded"): Promise<WindowPolicy> {
    return JSON.parse(this.client.setWindowMode(mode, windowNonce)) as WindowPolicy;
  }

  async exportTranscript(format: "markdown" | "plain" | "csv", path: string): Promise<string> {
    return this.client.export(format, path);
  }

  async listSources(): Promise<AudioSourceInfo[]> {
    return JSON.parse(this.client.listAudioSources()) as AudioSourceInfo[];
  }

  async applyRuntimeConfig(layer: unknown): Promise<RuntimeStatus> {
    return JSON.parse(this.client.applyRuntimeConfig(JSON.stringify(layer))) as RuntimeStatus;
  }

  async applyScenario(scenario: string): Promise<RuntimeStatus> {
    return JSON.parse(this.client.applyScenario(scenario)) as RuntimeStatus;
  }

  async probeBackends(): Promise<BackendProbe> {
    return JSON.parse(this.client.probeBackends()) as BackendProbe;
  }

  async calibrate(): Promise<RuntimeStatus> {
    return JSON.parse(this.client.calibrate()) as RuntimeStatus;
  }
}

interface DemoLine {
  text: string;
  simplified: string;
}

const DEMO: Record<string, DemoLine[]> = {
  "fr-FR": [
    {
      text: "Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera.",
      simplified: "Viens nous voir bientôt. Viens quand tu peux.",
    },
    {
      text: "J'ai été agréablement surprise par la tournure qu'ont prise les négociations.",
      simplified: "Les discussions se sont bien passées. Je suis contente.",
    },
    {
      text: "On s'est retrouvés coincés dans les embouteillages pendant une éternité.",
      simplified: "Il y avait beaucoup de voitures. On a attendu très longtemps.",
    },
    {
      text: "La conférence a été annulée par les organisateurs en raison des intempéries.",
      simplified: "Les organisateurs ont annulé la conférence. Il faisait très mauvais temps.",
    },
  ],
  "de-DE": [
    {
      text: "Die Veranstaltung musste leider aufgrund unvorhergesehener Umstände abgesagt werden.",
      simplified: "Wir haben die Veranstaltung abgesagt. Es gab ein Problem.",
    },
    {
      text: "Ich wäre dir sehr dankbar, wenn du mir die Unterlagen zukommen lassen könntest.",
      simplified: "Bitte schick mir die Unterlagen.",
    },
    {
      text: "Wegen des Geburtstags meiner Großmutter können wir erst später kommen.",
      simplified: "Meine Oma hat Geburtstag. Wir kommen später.",
    },
  ],
  "en-US": [
    {
      text: "We would appreciate it if you could send us the documents before the meeting.",
      simplified: "Please send us the documents before the meeting.",
    },
    {
      text: "The project was delayed because the budget was not approved in time.",
      simplified: "The project was late. The budget was not ready.",
    },
  ],
  "es-ES": [
    {
      text: "Nos gustaría que pudieras venir a la reunión la semana que viene.",
      simplified: "Ven a la reunión la semana que viene. Nos gustaría verte.",
    },
    {
      text: "La conferencia fue cancelada por los organizadores debido al mal tiempo.",
      simplified: "Los organizadores cancelaron la conferencia. Hacía mal tiempo.",
    },
  ],
};

class DemoTransport implements Transport {
  readonly kind = "demo" as const;
  private seq = 0;
  private mode: SimplificationMode = "on_demand";
  private language = "fr-FR";
  private segments: Array<{
    id: string;
    text: string;
    simplified: string | null;
    startMs: number;
    endMs: number;
  }> = [];
  private started = 0;

  constructor(private onEvent: (envelope: Envelope) => void) {}

  private emit(event: Envelope["event"]) {
    this.seq += 1;
    this.onEvent({ v: 1, seq: this.seq, ts: performance.now(), event });
  }

  async start(options: StartOptions): Promise<void> {
    this.mode = options.mode;
    this.language = options.language;
    this.segments = [];
    this.seq = 0;
    this.started = performance.now();
    this.emit({ type: "state", state: "listening", session_id: "demo-session" });
  }

  async feedDemo(): Promise<void> {
    const lines = DEMO[this.language] ?? DEMO["en-US"]!;
    let time = 0;
    for (const [index, line] of lines.entries()) {
      time += 2200;
      const id = `demo-${index + 1}`;
      this.segments.push({
        id,
        text: line.text,
        simplified: this.mode === "live" ? line.simplified : null,
        startMs: time,
        endMs: time + 1900,
      });
      this.emit({ type: "segment", id, text: line.text, start_ms: time, end_ms: time + 1900 });
      if (this.mode === "live") {
        this.emit({
          type: "simplified",
          id,
          text: line.simplified,
          status: "clean",
          flags: [],
          latency_ms: 420,
        });
      }
    }
    this.emit({ type: "reversion", active: false, count: 0 });
  }

  async stop(): Promise<SessionSummary> {
    this.emit({ type: "state", state: "summary", session_id: "demo-session" });
    return {
      sessionId: "demo-session",
      language: this.language,
      segments: this.segments.length,
      simplified: this.segments.filter((segment) => segment.simplified).length,
      durationS: (performance.now() - this.started) / 1000,
      overrunCount: 0,
      lastSeq: this.seq,
    };
  }

  async pause(): Promise<void> {
    this.emit({ type: "state", state: "paused", session_id: "demo-session" });
  }

  async resume(): Promise<void> {
    this.emit({ type: "state", state: "listening", session_id: "demo-session" });
  }

  async simplify(id: string): Promise<SimplifyResult> {
    const segment = this.segments.find((candidate) => candidate.id === id);
    const line = DEMO[this.language]?.[Number(id.split("-")[1]) - 1];
    if (!segment || !line) return { id, text: null, status: "verbatim", flags: [] };
    segment.simplified = line.simplified;
    this.emit({
      type: "simplified",
      id,
      text: line.simplified,
      status: "clean",
      flags: [],
      latency_ms: 400,
    });
    return { id, text: line.simplified, status: "clean", flags: [] };
  }

  async simplifyText(text: string): Promise<SimplifyResult> {
    const first = text.split(/[.!?]/)[0] ?? text;
    const words = first.split(/\s+/).slice(0, 14).join(" ");
    return { id: "typed", text: words, status: "clean", flags: [] };
  }

  async snapshot(): Promise<Snapshot> {
    return {
      state: "listening",
      lastSeq: this.seq,
      sessionId: "demo-session",
      segments: this.segments.map((segment) => ({
        id: segment.id,
        text: segment.text,
        simplified: segment.simplified,
        status: segment.simplified ? "clean" : "verbatim",
        flags: [],
        startMs: segment.startMs,
        endMs: segment.endMs,
        detectedLanguage: null,
      })),
      partial: null,
      reversionActive: false,
      overrunCount: 0,
      asrStatus: "simplify_only",
      language: this.language,
    };
  }

  async runtimeStatus(): Promise<RuntimeStatus> {
    return demoRuntime();
  }

  async setWindowMode(): Promise<WindowPolicy> {
    return { applied: false, backend: "demo", notes: ["native bridge unavailable"] };
  }

  async exportTranscript(_format: string, path: string): Promise<string> {
    return path;
  }

  async listSources(): Promise<AudioSourceInfo[]> {
    return [
      { id: "demo", label: "Demo conversation", kind: "demo", available: true },
      { id: "silence", label: "Synthetic silence", kind: "silence", available: true },
    ];
  }

  async applyRuntimeConfig(_layer: unknown): Promise<RuntimeStatus> {
    return demoRuntime();
  }

  async applyScenario(_scenario: string): Promise<RuntimeStatus> {
    return demoRuntime();
  }

  async probeBackends(): Promise<BackendProbe> {
    return {
      backend: "auto",
      devices: [
        { id: "cpu", kind: "cpu", available: true },
        { id: "vulkan", kind: "gpu", available: false },
      ],
      mockCalibration: true,
    };
  }

  async calibrate(): Promise<RuntimeStatus> {
    return demoRuntime();
  }
}

function demoRuntime(): RuntimeStatus {
  return {
    requestedBackend: "auto",
    effectiveBackend: "cpu",
    requestedDevice: "auto",
    effectiveDevice: "cpu",
    requestedGpuLayers: "auto",
    effectiveGpuLayers: 0,
    loadedBackendModules: ["cpu"],
    cpuVariant: "avx2",
    uiBackend: "vulkan",
    windowPolicy: "console",
    model: "qwen3-0.6b-easylanguage",
    quant: "q4_k_m",
    modelHash: null,
    ttftMs: null,
    tokensPerSec: null,
    lastCalibration: null,
    fallbackReason: null,
    strict: false,
    eventDelivery: { lastSeq: 0, lastEventAgeMs: 0, dropped: 0 },
  };
}
