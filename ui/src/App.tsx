import { mkdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { useGpuixRequired } from "@gpuix/react";
import { useEffect, useRef, useState } from "react";
import { AuroraField, Toast } from "./aurora/components";
import { colors } from "./aurora/theme";
import { WindowChrome } from "./aurora/WindowChrome";
import {
  type AudioSourceInfo,
  createTransport,
  readWorkArea,
  type SimplifyResult,
  type Transport,
} from "./bridge/transport";
import { languageByCode } from "./i18n/languages";
import { deviceLocales, resolveLocale, t } from "./i18n/locales";
import { exeDir } from "./packaged";
import { ConsoleScreen } from "./screens/Console";
import { LiveScreen } from "./screens/Live";
import { Onboarding } from "./screens/Onboarding";
import { OverlayScreen } from "./screens/Overlay";
import { SettingsScreen } from "./screens/Settings";
import { SummaryScreen } from "./screens/Summary";
import { exportDir, exportPath, planSession } from "./sessionPlan";
import { defaultSettings, loadSettings, saveSettings, type UiSettings } from "./settings";
import { createModelStore, useModels } from "./store/modelStore";
import { createSessionStore, useSession } from "./store/sessionStore";
import { sizeForMode } from "./windowSize";

const hasSizeOverride = () => Boolean(process.env.LL_WIDTH || process.env.LL_HEIGHT);

type Screen = "onboarding" | "console" | "live" | "settings" | "summary";
type WindowMode = "console" | "overlay" | "expanded";

const AUTO_SCREENS = new Set<Screen>(["live"]);

function readScreen(): Screen {
  const raw = (process.env.LL_SCREEN ?? "").toLowerCase();
  if (raw === "overlay" || raw === "expanded") return "live";
  if (
    raw === "onboarding" ||
    raw === "console" ||
    raw === "live" ||
    raw === "settings" ||
    raw === "summary"
  ) {
    return raw;
  }
  return "onboarding";
}

function readWindowMode(): WindowMode {
  const raw = (process.env.LL_SCREEN ?? "").toLowerCase();
  if (raw === "overlay") return "overlay";
  if (raw === "expanded") return "expanded";
  return "console";
}

function readLanguage(): string {
  const code = process.env.LL_LANGUAGE ?? "fr-FR";
  return languageByCode(code).code;
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export function App() {
  const initialLanguage = readLanguage();
  const [bootStore] = useState(() => {
    const store = createSessionStore(
      initialLanguage,
      process.env.LL_UI_LOCALE ?? languageByCode(initialLanguage).uiCode,
    );
    store.setMode(process.env.LL_MODE === "live" ? "live" : "on_demand");
    return store;
  });
  const state = useSession(bootStore);
  const [screen, setScreen] = useState<Screen>(readScreen());
  const [windowMode, setWindowModeState] = useState<WindowMode>(readWindowMode());
  const [backTo, setBackTo] = useState<Screen>("console");
  const [settings, setSettings] = useState<UiSettings>(loadSettings);
  const [toast, setToast] = useState<string | null>(null);
  const [simplifyingId, setSimplifyingId] = useState<string | null>(null);
  const [modelStore] = useState(createModelStore);
  const models = useModels(modelStore);
  const [sources, setSources] = useState<AudioSourceInfo[] | null>(null);
  const transportRef = useRef<Transport | null>(null);
  const started = useRef(false);
  const renderer = useGpuixRequired();

  const applySettings = (patch: Partial<UiSettings>) => {
    setSettings((current) => {
      const next = { ...current, ...patch };
      saveSettings(next);
      return next;
    });
  };

  const deviceUi = resolveLocale(false, null, deviceLocales());
  const locale =
    process.env.LL_UI_LOCALE ??
    (settings.uiLanguage === "device" ? deviceUi : languageByCode(state.language).uiCode);

  useEffect(() => {
    // Register the UI faces GPUIX may not list on its own. Missing files are
    // fine: the platform font is the fallback.
    const bundled = join(import.meta.dir, "..", "assets", "fonts", "MPLUSRounded1c-Medium.ttf");
    const packaged = join(exeDir(process.execPath), "fonts", "MPLUSRounded1c-Medium.ttf");
    const candidates = [
      bundled,
      packaged,
      "C:\\Windows\\Fonts\\segoeui.ttf",
      "C:\\Windows\\Fonts\\seguisb.ttf",
      "C:\\Windows\\Fonts\\NotoSansArabic-Regular.ttf",
      "/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf",
      "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
    ];
    for (const path of candidates) {
      try {
        renderer.registerFont?.(readFileSync(path));
      } catch {
        // Optional face; the system fallback still renders.
      }
    }
  }, [renderer]);

  useEffect(() => {
    if (!simplifyingId) return;
    const segment = state.segments.find((item) => item.id === simplifyingId);
    if (segment?.simplified) {
      setSimplifyingId((current) => (current === simplifyingId ? null : current));
    }
  }, [simplifyingId, state.segments]);

  const ensureTransport = (): Transport => {
    if (!transportRef.current) {
      transportRef.current = createTransport((envelope) => bootStore.applyEnvelope(envelope));
    }
    return transportRef.current;
  };

  const refreshSources = async () => {
    try {
      setSources(await ensureTransport().listSources());
    } catch {
      // A missing addon or a failed enumeration leaves the picker unchanged;
      // starting still reports the real error.
      setSources(null);
    }
  };

  const stopWith = async (transport: Transport) => {
    try {
      const snapshot = await transport.snapshot();
      bootStore.replaceSnapshot(snapshot);
    } catch {
      // The summary is still useful if the snapshot is unavailable.
    }
    const summary = await transport.stop();
    bootStore.setSummary(summary);
    bootStore.setPhase("summary");
    setScreen("summary");
  };

  const startSession = async () => {
    const mode = process.env.LL_MODE === "live" ? "live" : state.mode;
    const plan = planSession(state.source, languageByCode(state.language).asrStatus);
    if (!plan.ok) {
      setToast(plan.error);
      return;
    }
    const source = plan.source;
    const language = state.language;
    const uiLocale = process.env.LL_UI_LOCALE ?? languageByCode(language).uiCode;
    bootStore.reset();
    bootStore.setLanguage(language, uiLocale);
    bootStore.setSource(source);
    bootStore.setMode(mode);
    bootStore.setPhase("listening");
    bootStore.setAsrStatus(plan.asrStatus);
    const transport = createTransport((envelope) => bootStore.applyEnvelope(envelope));
    transportRef.current = transport;
    setScreen("live");
    let deviceId =
      source === "monitor"
        ? settings.outputDeviceId || undefined
        : source === "mic"
          ? settings.inputDeviceId || undefined
          : undefined;
    if (deviceId) {
      // A saved endpoint may have been unplugged since it was chosen. Keep the
      // selection honest instead of failing the session on a dead id.
      const info = (await transport.listSources().catch(() => [] as AudioSourceInfo[])).find(
        (entry) => entry.id === source,
      );
      const devices = info?.devices ?? [];
      if (devices.length > 0 && !devices.some((entry) => entry.id === deviceId)) {
        deviceId = undefined;
        setToast(t(locale, "The selected device is unavailable; using the system default."));
      }
    }
    try {
      await transport.start({
        language,
        source,
        deviceId,
        mode,
        detection: settings.detection,
        scenario: settings.scenario,
      });
      if (plan.feedDemo) await transport.feedDemo();
      if (windowMode !== "console") {
        const policy = await transport.setWindowMode(windowMode);
        if (!policy.applied) setToast("Window policy unavailable; using a normal window.");
      }
    } catch (error) {
      bootStore.setError(error instanceof Error ? error.message : "Could not start the session");
    }
  };

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    const target = readScreen();
    if (AUTO_SCREENS.has(target)) {
      void startSession();
    } else {
      void ensureTransport()
        .setWindowMode(windowMode)
        .then(() => ensureTransport().runtimeStatus())
        .then((runtime) => {
          bootStore.setRuntime(runtime);
          if (runtime.ui?.degradedReason && runtime.ui.backend !== "demo") {
            setToast(`Software rendering: ${runtime.ui.degradedReason}`);
          }
        })
        .catch(() => undefined);
      if (target === "summary") {
        void startSession()
          .then(() => sleep(900))
          .then(() => {
            if (transportRef.current) void stopWith(transportRef.current);
          });
      }
    }
    // The boot screen is read once; later navigation is user-driven.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const setWindowMode = async (mode: WindowMode) => {
    setWindowModeState(mode);
    try {
      await ensureTransport().setWindowMode(mode);
      if (mode !== "overlay" && !hasSizeOverride()) {
        const size = sizeForMode(mode, readWorkArea());
        await ensureTransport().resizeWindow(size.width, size.height);
      }
    } catch {
      setToast("Window policy unavailable; using a normal window.");
    }
  };

  const refreshModels = async () => {
    try {
      modelStore.setStatus(await ensureTransport().modelStatus());
    } catch {
      // Status stays as it was; the download button still works.
    }
  };

  const onDownloadModels = async () => {
    await refreshModels();
    modelStore.begin();
    try {
      await ensureTransport().downloadModels((envelope) => {
        modelStore.applyEnvelope(envelope);
        if (envelope.event.type === "model_ready") void refreshModels();
      });
    } catch (error) {
      modelStore.fail({
        id: modelStore.getSnapshot().models[0]?.id ?? "",
        code: "UNAVAILABLE",
        reason: error instanceof Error ? error.message : "download unavailable",
      });
    }
  };

  const onCancelDownload = () => {
    void ensureTransport()
      .cancelDownload()
      .catch(() => false);
  };

  useEffect(() => {
    void refreshModels();
    void refreshSources();
    // Model and endpoint status is read once at start; downloads and the
    // sound menu refresh them themselves.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onLanguage = (code: string) => {
    const option = languageByCode(code);
    bootStore.setLanguage(option.code, option.uiCode);
    if (state.phase === "listening") {
      setToast(t(locale, "Language change applies to the next session."));
    }
  };

  const onDevice = (deviceId: string) => {
    if (state.source === "monitor") {
      applySettings({ outputDeviceId: deviceId });
    } else if (state.source === "mic") {
      applySettings({ inputDeviceId: deviceId });
    }
  };

  const onSimplify = async (id: string) => {
    setSimplifyingId(id);
    try {
      const result = await ensureTransport().simplify(id);
      if (result.status === "queued") {
        // The bridge runs the LLM on a worker and emits `simplified`; the
        // effect above clears the spinner. The timeout only un-sticks a row
        // when the request never produces a result.
        setTimeout(() => {
          setSimplifyingId((current) => (current === id ? null : current));
        }, 15000);
      } else {
        bootStore.applySimplified(result);
        setSimplifyingId((current) => (current === id ? null : current));
      }
    } catch {
      setToast(t(locale, "Could not simplify that sentence."));
      setSimplifyingId((current) => (current === id ? null : current));
    }
  };

  const onTypedSimplify = async (text: string): Promise<SimplifyResult> => {
    try {
      return await ensureTransport().simplifyText(text);
    } catch {
      return { id: "typed", text: null, status: "verbatim", flags: [] };
    }
  };

  const onExport = async (format: "markdown" | "plain" | "csv") => {
    const path = exportPath(format, new Date(), homedir());
    try {
      mkdirSync(exportDir(homedir()), { recursive: true });
      await ensureTransport().exportTranscript(format, path);
      setToast(`${t(locale, "Export transcript")} · ${path}`);
    } catch {
      setToast(`${t(locale, "Export transcript")} · failed: ${exportDir(homedir())}`);
    }
  };

  const onScenario = async (scenario: string) => {
    setSettings((current) => ({ ...current, scenario }));
    try {
      const runtime = await ensureTransport().applyScenario(scenario);
      bootStore.setRuntime(runtime);
      setToast(`Scenario: ${scenario}`);
    } catch {
      setToast(`Scenario ${scenario} is unavailable on this machine.`);
    }
  };

  const onProbe = async () => {
    try {
      const probe = await ensureTransport().probeBackends();
      const available = probe.devices
        .filter((device) => device.available)
        .map((device) => device.id)
        .join(", ");
      const parts = [available || "no accelerators"];
      if (probe.ui) {
        parts.push(
          `${probe.ui.backend}: ${probe.ui.hardware ? "GPU" : "software"}${probe.ui.adapter ? ` (${probe.ui.adapter})` : ""}`,
        );
      }
      if (probe.cudaDevices && probe.cudaDevices.length > 0) {
        parts.push(`CUDA: ${probe.cudaDevices.map((device) => device.name).join(", ")}`);
      }
      setToast(`Backends: ${parts.join(" · ")}`);
    } catch {
      setToast("Backend probe failed.");
    }
  };

  const onCalibrate = async () => {
    try {
      const runtime = await ensureTransport().calibrate();
      bootStore.setRuntime(runtime);
      setToast("Calibration complete (mock until a GGUF is installed).");
    } catch {
      setToast("Calibration failed.");
    }
  };

  const openSettings = () => {
    // Settings needs a normal window; the overlay is 780x250 and cannot show it.
    if (windowMode === "overlay") void setWindowMode("console");
    setBackTo(screen === "settings" ? "console" : screen);
    setScreen("settings");
  };

  const screenContent = (() => {
    if (screen === "onboarding") {
      return (
        <Onboarding
          language={state.language}
          locale={locale}
          onLanguage={onLanguage}
          onFinish={() => setScreen("console")}
          models={models}
          onDownload={() => void onDownloadModels()}
          onCancelDownload={onCancelDownload}
        />
      );
    }
    if (screen === "console") {
      return (
        <ConsoleScreen
          language={state.language}
          locale={locale}
          source={state.source}
          models={models}
          hasTranscript={state.segments.length > 0}
          sources={sources}
          deviceId={
            state.source === "monitor"
              ? settings.outputDeviceId
              : state.source === "mic"
                ? settings.inputDeviceId
                : ""
          }
          onLanguage={onLanguage}
          onSource={(source) => bootStore.setSource(source)}
          onDevice={onDevice}
          onRefreshSources={() => void refreshSources()}
          onStart={() => void startSession()}
          onSettings={openSettings}
          onSummary={() => setScreen(state.segments.length > 0 ? "summary" : "console")}
        />
      );
    }
    if (screen === "settings") {
      return (
        <SettingsScreen
          locale={locale}
          language={state.language}
          settings={settings}
          onChange={applySettings}
          runtime={state.runtime}
          onBack={() => setScreen(backTo)}
          onProbe={onProbe}
          onCalibrate={onCalibrate}
          onReload={async () => setToast("Model reload is a no-op in the mock backend.")}
          onClearCache={async () => setToast("Prefix cache cleared.")}
          onCopyDiagnostics={() =>
            setToast("Diagnostics are content-free; use Export → Diagnostics CSV.")
          }
          onScenario={(scenario) => void onScenario(scenario)}
          models={models}
          onDownloadModels={() => void onDownloadModels()}
          onReset={() => {
            applySettings(defaultSettings);
            void onScenario("auto");
          }}
        />
      );
    }
    if (screen === "summary") {
      return (
        <SummaryScreen
          state={state}
          locale={locale}
          onExport={onExport}
          onDone={() => {
            void setWindowMode("console");
            setScreen("console");
          }}
        />
      );
    }
    if (windowMode === "overlay") {
      return (
        <OverlayScreen
          state={state}
          locale={locale}
          captionSize={settings.captionSize}
          showTier1={settings.showTier1}
          onSimplify={(id) => void onSimplify(id)}
          onPause={() => void ensureTransport().pause()}
          onResume={() => void ensureTransport().resume()}
          onSettings={openSettings}
          onDrag={() => void ensureTransport().beginDrag()}
          onExpand={() => {
            void setWindowMode("expanded");
            setScreen("live");
          }}
          onConsole={() => {
            void setWindowMode("console");
            setScreen("live");
          }}
        />
      );
    }
    return (
      <LiveScreen
        state={state}
        locale={locale}
        captionSize={settings.captionSize}
        showTier1={settings.showTier1}
        windowMode={windowMode}
        onSimplify={(id) => void onSimplify(id)}
        onStop={() => void stopWith(ensureTransport())}
        onPause={() => void ensureTransport().pause()}
        onResume={() => void ensureTransport().resume()}
        onSettings={openSettings}
        onWindowMode={(mode) => void setWindowMode(mode)}
        onTypedSimplify={onTypedSimplify}
        compact={false}
        simplifyingId={simplifyingId}
      />
    );
  })();

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        position: "relative",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        backgroundColor: colors.bg,
      }}
    >
      <AuroraField active={screen === "live"} />
      {windowMode === "overlay" ? null : (
        <WindowChrome
          onDrag={() => void ensureTransport().beginDrag()}
          onMinimize={() => {
            try {
              renderer.minimizeWindow?.();
            } catch {
              // No native window in the renderer harness.
            }
          }}
          onToggleMax={() => {
            try {
              renderer.zoomWindow?.();
            } catch {
              // No native window in the renderer harness.
            }
          }}
          onClose={() => {
            void ensureTransport()
              .closeWindow()
              .then((closed) => {
                if (!closed) process.exit(0);
              })
              .catch(() => process.exit(0));
          }}
        />
      )}
      <div style={{ position: "relative", flexGrow: 1, minHeight: 0, width: "100%" }}>
        {screenContent}
      </div>
      {toast ? (
        <div
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: 14,
            display: "flex",
            flexDirection: "row",
            justifyContent: "center",
          }}
        >
          <div
            role="button"
            tabIndex={0}
            aria-label="Dismiss notification"
            onClick={() => setToast(null)}
            onKeyDown={(event) => {
              if (event.key === "enter" || event.key === "space" || event.key === " ")
                setToast(null);
            }}
            style={{ cursor: "pointer" }}
          >
            <Toast message={toast} tone="info" />
          </div>
        </div>
      ) : null}
    </div>
  );
}
