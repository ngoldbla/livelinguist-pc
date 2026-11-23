import { readFileSync } from "node:fs";
import { useGpuixRequired } from "@gpuix/react";
import { useEffect, useRef, useState } from "react";
import { AuroraField, Toast } from "./aurora/components";
import { colors } from "./aurora/theme";
import { createTransport, type SimplifyResult, type Transport } from "./bridge/transport";
import { languageByCode } from "./i18n/languages";
import { t } from "./i18n/locales";
import { ConsoleScreen } from "./screens/Console";
import { LicensesScreen } from "./screens/Licenses";
import { LiveScreen } from "./screens/Live";
import { Onboarding } from "./screens/Onboarding";
import { OverlayScreen } from "./screens/Overlay";
import { SettingsScreen, type UiSettings } from "./screens/Settings";
import { SummaryScreen } from "./screens/Summary";
import { createSessionStore, useSession } from "./store/sessionStore";

type Screen = "onboarding" | "console" | "live" | "settings" | "summary" | "licenses";
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
    raw === "summary" ||
    raw === "licenses"
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

const defaultSettings: UiSettings = {
  captionSize: 20,
  showTier1: true,
  detection: true,
  reducedEffects: false,
  followLearning: true,
  devEnabled: true,
  scenario: "mock",
};

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
  const [settings, setSettings] = useState<UiSettings>(defaultSettings);
  const [toast, setToast] = useState<string | null>(null);
  const transportRef = useRef<Transport | null>(null);
  const started = useRef(false);
  const renderer = useGpuixRequired();

  const locale =
    process.env.LL_UI_LOCALE ??
    (settings.followLearning ? languageByCode(state.language).uiCode : state.uiLocale || "en");

  useEffect(() => {
    try {
      const bytes = readFileSync("/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf");
      renderer.registerFont?.(bytes);
    } catch {
      // System fallback is fine; Arabic still renders through Noto Sans Arabic.
    }
  }, [renderer]);

  const ensureTransport = (): Transport => {
    if (!transportRef.current) {
      transportRef.current = createTransport((envelope) => bootStore.applyEnvelope(envelope));
    }
    return transportRef.current;
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
    const source = state.source === "mic" || state.source === "monitor" ? "demo" : state.source;
    const language = state.language;
    const uiLocale = process.env.LL_UI_LOCALE ?? languageByCode(language).uiCode;
    bootStore.reset();
    bootStore.setLanguage(language, uiLocale);
    bootStore.setSource(source);
    bootStore.setMode(mode);
    bootStore.setPhase("listening");
    bootStore.setAsrStatus("simplify_only");
    const transport = createTransport((envelope) => bootStore.applyEnvelope(envelope));
    transportRef.current = transport;
    setScreen("live");
    try {
      await transport.start({
        language,
        source,
        mode,
        detection: settings.detection,
        scenario: settings.scenario,
      });
      await transport.feedDemo();
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
    } catch {
      setToast("Window policy unavailable; using a normal window.");
    }
  };

  const onLanguage = (code: string) => {
    const option = languageByCode(code);
    bootStore.setLanguage(option.code, option.uiCode);
    if (state.phase === "listening") {
      setToast(t(locale, "Language change applies to the next session."));
    }
  };

  const onSimplify = async (id: string) => {
    try {
      const result = await ensureTransport().simplify(id);
      bootStore.applySimplified(result);
    } catch {
      setToast(t(locale, "Could not simplify that sentence."));
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
    const extension = format === "markdown" ? "md" : format === "plain" ? "txt" : "csv";
    const path = `/tmp/livelinguist-export-${Date.now()}.${extension}`;
    await ensureTransport().exportTranscript(format, path);
    setToast(`${t(locale, "Export transcript")} · ${path}`);
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
    setBackTo(screen === "settings" ? "console" : screen);
    setScreen("settings");
  };

  const openLicenses = () => {
    setBackTo(screen === "licenses" ? "console" : screen);
    setScreen("licenses");
  };

  const screenContent = (() => {
    if (screen === "onboarding") {
      return (
        <Onboarding
          language={state.language}
          locale={locale}
          onLanguage={onLanguage}
          onFinish={() => setScreen("console")}
          onOpenLicenses={openLicenses}
        />
      );
    }
    if (screen === "console") {
      return (
        <ConsoleScreen
          language={state.language}
          locale={locale}
          source={state.source}
          mode={state.mode}
          onLanguage={onLanguage}
          onSource={(source) => bootStore.setSource(source)}
          onMode={(mode) => bootStore.setMode(mode)}
          onStart={() => void startSession()}
          onSettings={openSettings}
          onLicenses={openLicenses}
          onSummary={() => setScreen(state.segments.length > 0 ? "summary" : "console")}
        />
      );
    }
    if (screen === "settings") {
      return (
        <SettingsScreen
          locale={locale}
          settings={settings}
          onChange={(patch) => setSettings((current) => ({ ...current, ...patch }))}
          runtime={state.runtime}
          onBack={() => setScreen(backTo)}
          onLicenses={openLicenses}
          onProbe={onProbe}
          onCalibrate={onCalibrate}
          onReload={async () => setToast("Model reload is a no-op in the mock backend.")}
          onClearCache={async () => setToast("Prefix cache cleared.")}
          onCopyDiagnostics={() =>
            setToast("Diagnostics are content-free; use Export → Diagnostics CSV.")
          }
          onScenario={(scenario) => void onScenario(scenario)}
          onReset={() => {
            setSettings(defaultSettings);
            void onScenario("auto");
          }}
        />
      );
    }
    if (screen === "licenses") {
      return <LicensesScreen locale={locale} onBack={() => setScreen(backTo)} />;
    }
    if (screen === "summary") {
      return (
        <SummaryScreen
          state={state}
          locale={locale}
          onExport={onExport}
          onBack={() => setScreen("console")}
          onConsole={() => {
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
        onSimplify={(id) => void onSimplify(id)}
        onStop={() => void stopWith(ensureTransport())}
        onPause={() => void ensureTransport().pause()}
        onResume={() => void ensureTransport().resume()}
        onSettings={openSettings}
        onMode={(mode) => bootStore.setMode(mode)}
        onWindowMode={(mode) => void setWindowMode(mode)}
        onTypedSimplify={onTypedSimplify}
        compact={false}
      />
    );
  })();

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        position: "relative",
        overflow: "hidden",
        backgroundColor: colors.bg,
      }}
    >
      <AuroraField active={screen === "live" && !settings.reducedEffects} />
      <div style={{ position: "relative", width: "100%", height: "100%" }}>{screenContent}</div>
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
