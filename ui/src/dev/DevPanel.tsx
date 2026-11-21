import { useState } from "react";
import {
  Badge,
  Button,
  Divider,
  SectionHeader,
  Surface,
  Toast,
  Toggle,
} from "../aurora/components";
import { colors, textStyle } from "../aurora/theme";
import type { RuntimeStatus } from "../bridge/generated/RuntimeStatus";
import { t } from "../i18n/locales";

const SCENARIOS = [
  "auto",
  "cpu-strict",
  "no-gpu",
  "vulkan-strict",
  "tier-c",
  "mock",
  "stress",
  "asr-only",
  "llm-only",
  "simplify-only",
] as const;

export function DevPanel({
  locale,
  runtime,
  scenario,
  onScenario,
  onProbe,
  onCalibrate,
  onReload,
  onClearCache,
  onCopyDiagnostics,
  onReset,
}: {
  locale: string;
  runtime: RuntimeStatus | null;
  scenario: string;
  onScenario: (scenario: string) => void;
  onProbe: () => Promise<void>;
  onCalibrate: () => Promise<void>;
  onReload: () => Promise<void>;
  onClearCache: () => Promise<void>;
  onCopyDiagnostics: () => void;
  onReset: () => void;
}) {
  const [toast, setToast] = useState<string | null>(null);
  const [strict, setStrict] = useState(runtime?.strict ?? false);

  const wrap = async (label: string, action: () => Promise<void>) => {
    await action();
    setToast(label);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
      <SectionHeader
        eyebrow={t(locale, "DEVELOPER")}
        title={t(locale, "Runtime health")}
        subtitle={t(
          locale,
          "Requested vs effective backend, calibration, and event stats. Content-free.",
        )}
      />
      <div style={{ display: "flex", flexDirection: "row", gap: 14 }}>
        <Surface
          style={{ padding: 16, flexGrow: 1, display: "flex", flexDirection: "column", gap: 8 }}
        >
          <text style={{ ...textStyle(10, colors.faint, 700) }}>
            {t(locale, "SCENARIO").toUpperCase()}
          </text>
          <div style={{ display: "flex", flexDirection: "row", gap: 6, flexWrap: "wrap" }}>
            {SCENARIOS.map((id) => (
              <Button
                key={id}
                label={id}
                kind={scenario === id ? "primary" : "secondary"}
                onClick={() => onScenario(id)}
                style={{ height: 30, paddingLeft: 10, paddingRight: 10 }}
              />
            ))}
          </div>
          <Divider />
          <Toggle
            label={t(locale, "Strict mode")}
            checked={strict}
            onChange={setStrict}
            description="Fail loudly instead of falling back."
          />
          <div style={{ display: "flex", flexDirection: "row", gap: 8, marginTop: 4 }}>
            <Button
              label={t(locale, "Probe backends")}
              kind="secondary"
              onClick={() => void wrap(t(locale, "Backends probed"), onProbe)}
            />
            <Button
              label={t(locale, "Run calibration")}
              kind="secondary"
              onClick={() => void wrap(t(locale, "Calibration complete"), onCalibrate)}
            />
          </div>
          <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
            <Button
              label={t(locale, "Reload model")}
              kind="ghost"
              onClick={() => void wrap(t(locale, "Model reload requested"), onReload)}
            />
            <Button
              label={t(locale, "Clear KV cache")}
              kind="ghost"
              onClick={() => void wrap(t(locale, "KV cache cleared"), onClearCache)}
            />
            <Button
              label={t(locale, "Copy diagnostics")}
              kind="ghost"
              onClick={onCopyDiagnostics}
            />
          </div>
          <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
            <Button label={t(locale, "Reset overrides")} kind="danger" onClick={onReset} />
          </div>
        </Surface>

        <Surface
          style={{
            width: 360,
            flexShrink: 0,
            padding: 16,
            display: "flex",
            flexDirection: "column",
            gap: 7,
          }}
        >
          <text style={{ ...textStyle(10, colors.faint, 700) }}>
            {t(locale, "RUNTIME").toUpperCase()}
          </text>
          <Health
            label={t(locale, "Requested")}
            value={`${runtime?.requestedBackend ?? "auto"} / ${runtime?.requestedDevice ?? "auto"} / ${runtime?.requestedGpuLayers ?? "auto"}`}
          />
          <Health
            label={t(locale, "Effective")}
            value={`${runtime?.effectiveBackend ?? "cpu"} / ${runtime?.effectiveDevice ?? "cpu"} / ${runtime?.effectiveGpuLayers ?? 0}`}
          />
          <Health
            label={t(locale, "Backend")}
            value={runtime?.loadedBackendModules.join(", ") ?? "cpu"}
          />
          <Health label="CPU" value={runtime?.cpuVariant ?? "unknown"} />
          <Health
            label={t(locale, "Model")}
            value={`${runtime?.model ?? "qwen3-0.6b"} · ${runtime?.quant ?? "q4_k_m"}`}
          />
          <Health label={t(locale, "Model hash")} value={runtime?.modelHash ?? "unpinned"} />
          <Health
            label={t(locale, "TTFT")}
            value={runtime?.ttftMs == null ? "not measured" : `${runtime.ttftMs.toFixed(1)} ms`}
          />
          <Health
            label={t(locale, "Tokens/s")}
            value={runtime?.tokensPerSec == null ? "not measured" : String(runtime.tokensPerSec)}
          />
          <Health
            label={t(locale, "Event seq")}
            value={String(runtime?.eventDelivery.lastSeq ?? 0)}
          />
          <Health
            label={t(locale, "Dropped")}
            value={String(runtime?.eventDelivery.dropped ?? 0)}
          />
          <Health
            label={t(locale, "Last event age")}
            value={`${runtime?.eventDelivery.lastEventAgeMs ?? 0} ms`}
          />
          <Health label={t(locale, "Strict")} value={String(runtime?.strict ?? false)} />
          <Health
            label={t(locale, "Last calibration")}
            value={runtime?.lastCalibration ?? "none"}
          />
          <Health
            label="Window"
            value={`${runtime?.windowPolicy ?? "console"} · ${runtime?.uiBackend ?? "vulkan"}`}
          />
          <Badge label={t(locale, "Content-free diagnostics")} tone="mint" />
        </Surface>
      </div>
      {toast ? <Toast message={toast} tone="mint" /> : null}
    </div>
  );
}

function Health({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 8,
        paddingTop: 3,
        paddingBottom: 3,
      }}
    >
      <text style={{ ...textStyle(10, colors.faint), width: 100 }}>{label}</text>
      <text style={{ ...textStyle(10, colors.muted), flexGrow: 1, minWidth: 0 }}>{value}</text>
    </div>
  );
}
