import { useState } from "react";
import {
  Badge,
  Button,
  Divider,
  PageHeader,
  SectionHeader,
  Slider,
  StatusDot,
  Surface,
  Tabs,
  Toggle,
} from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
import type { RuntimeStatus } from "../bridge/generated/RuntimeStatus";
import { DevPanel } from "../dev/DevPanel";
import { t } from "../i18n/locales";
import { CAPTION_SIZE_MAX, CAPTION_SIZE_MIN, type UiSettings } from "../settings";
import { allReady, type ModelView, percent } from "../store/modelStore";
import { APP_VERSION } from "../version";

export function SettingsScreen({
  locale,
  settings,
  onChange,
  runtime,
  onBack,
  onLicenses,
  onProbe,
  onCalibrate,
  onReload,
  onClearCache,
  onCopyDiagnostics,
  onScenario,
  onReset,
  models,
  onDownloadModels,
}: {
  locale: string;
  settings: UiSettings;
  onChange: (patch: Partial<UiSettings>) => void;
  runtime: RuntimeStatus | null;
  onBack: () => void;
  onLicenses: () => void;
  onProbe: () => Promise<void>;
  onCalibrate: () => Promise<void>;
  onReload: () => Promise<void>;
  onClearCache: () => Promise<void>;
  onCopyDiagnostics: () => void;
  onScenario: (scenario: string) => void;
  onReset: () => void;
  models?: ModelView;
  onDownloadModels?: () => void;
}) {
  const [tab, setTab] = useState(process.env.LL_TAB ?? "captions");
  const anyMissing = models ? !allReady(models) : true;

  const modelState = (id: string, size: string) => {
    const model = models?.models.find((entry) => entry.id === id);
    if (!model) return `${size} · ${t(locale, "Not downloaded")}`;
    if (model.phase === "ready") return t(locale, "Ready");
    if (model.phase === "downloading") return `${percent(model)}%`;
    return model.present ? t(locale, "Download failed: %@", "retry") : t(locale, "Not downloaded");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, padding: 24, height: "100%" }}>
      <PageHeader
        backLabel={t(locale, "Back")}
        title={t(locale, "Settings")}
        subtitle={t(locale, "Changes apply immediately and are kept for next time.")}
        onBack={onBack}
      />

      <Tabs
        tabs={[
          { id: "captions", label: t(locale, "Captions") },
          { id: "models", label: t(locale, "Models") },
          { id: "developer", label: t(locale, "Developer") },
          { id: "about", label: t(locale, "About") },
        ]}
        value={tab}
        onChange={setTab}
      />

      <div
        style={{
          flexGrow: 1,
          minHeight: 0,
          overflow: "scroll",
          scrollbarColor: "#FFFFFF22",
          scrollbarWidth: 6,
        }}
      >
        {tab === "captions" ? (
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
            <Surface
              style={{
                width: "100%",
                maxWidth: 780,
                padding: 22,
                display: "flex",
                flexDirection: "column",
                gap: 14,
              }}
            >
              <SectionHeader
                title={t(locale, "Captions")}
                subtitle={t(locale, "Caption size and the two-tier display.")}
              />
              <div
                style={{
                  padding: 12,
                  backgroundColor: "#00000033",
                  borderRadius: 8,
                  borderWidth: 1,
                  borderColor: colors.border,
                  display: "flex",
                  flexDirection: "column",
                  gap: 8,
                }}
              >
                <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
                  {t(locale, "PREVIEW").toUpperCase()}
                </text>
                <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 20 }}>
                  On s'est retrouvés coincés dans les embouteillages pendant une éternité.
                </text>
                <text
                  style={{
                    ...textStyle(settings.captionSize, colors.text, 600),
                    lineHeight: Math.round(settings.captionSize * 1.35),
                  }}
                >
                  Il y avait beaucoup de voitures. On a attendu très longtemps.
                </text>
              </div>
              <Slider
                label={t(locale, "Caption size")}
                value={settings.captionSize}
                min={CAPTION_SIZE_MIN}
                max={CAPTION_SIZE_MAX}
                unit="px"
                onChange={(value) => onChange({ captionSize: value })}
              />
              <Divider />
              <div style={{ display: "flex", flexDirection: "row", gap: 20 }}>
                <div style={{ flexGrow: 1, minWidth: 0 }}>
                  <Toggle
                    label={t(locale, "Show live verbatim text (Tier 1)")}
                    checked={settings.showTier1}
                    onChange={(checked) => onChange({ showTier1: checked })}
                    description={t(
                      locale,
                      "Two caption tiers: verbatim partial + simplified final.",
                    )}
                  />
                </div>
                <div style={{ flexGrow: 1, minWidth: 0 }}>
                  <Toggle
                    label={t(locale, "Detect language")}
                    checked={settings.detection}
                    onChange={(checked) => onChange({ detection: checked })}
                    description={t(
                      locale,
                      "Language detection powers the English-reversion indicator.",
                    )}
                  />
                </div>
              </div>
              <Toggle
                label={t(locale, "Reduced effects")}
                checked={settings.reducedEffects}
                onChange={(checked) => onChange({ reducedEffects: checked })}
                description={t(
                  locale,
                  "Turns off the background glow. The app has no animations and redraws only when something changes.",
                )}
              />
              <Divider />
              <Toggle
                label={t(locale, "App language follows learning language")}
                checked={settings.followLearning}
                onChange={(checked) => onChange({ followLearning: checked })}
                description={t(locale, "Switch back to the device language any time.")}
              />
              <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
                <text style={textStyle(type.body, colors.muted)}>{t(locale, "App language")}</text>
                <div style={{ flexGrow: 1 }} />
                <Badge label={settings.followLearning ? locale : "en"} tone="accent" />
              </div>
            </Surface>
          </div>
        ) : null}

        {tab === "models" ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 14, minHeight: "100%" }}>
            <Surface style={{ padding: 22, display: "flex", flexDirection: "column", gap: 14 }}>
              <SectionHeader
                title={t(locale, "On-device models")}
                subtitle={t(locale, "No models are bundled. Downloads are public and tokenless.")}
              />
              <ModelCard
                name="Qwen3 0.6B EasyLanguage"
                detail="GGUF Q4_K_M · Apache-2.0 · ~397 MB"
                state={modelState("llm-qwen3-0.6b-easylanguage", "~397 MB")}
                ready={models?.models.some(
                  (entry) => entry.id === "llm-qwen3-0.6b-easylanguage" && entry.phase === "ready",
                )}
              />
              <ModelCard
                name="Nemotron 3.5 ASR 0.6B"
                detail="560 ms · int8 · 40 locales · ~682 MB"
                state={modelState("asr-nemotron-3.5-0.6b-560ms", "~682 MB")}
                ready={models?.models.some(
                  (entry) => entry.id === "asr-nemotron-3.5-0.6b-560ms" && entry.phase === "ready",
                )}
              />
              <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
                {anyMissing ? (
                  <Button
                    label={
                      models?.downloading
                        ? t(locale, "Downloading models…")
                        : t(locale, "Download models")
                    }
                    disabled={!onDownloadModels || models?.downloading}
                    onClick={() => onDownloadModels?.()}
                  />
                ) : (
                  <StatusDot tone="mint" label={t(locale, "Models ready")} />
                )}
                <Button label={t(locale, "Licenses")} kind="secondary" onClick={onLicenses} />
              </div>
            </Surface>
            <Surface style={{ padding: 16 }}>
              <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
                {t(
                  locale,
                  "Model downloads contact Hugging Face. Audio and transcripts stay on this device. Nothing is uploaded.",
                )}
              </text>
            </Surface>
          </div>
        ) : null}

        {tab === "developer" ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            <Surface style={{ padding: 16 }}>
              <Toggle
                label={t(locale, "Developer settings")}
                checked={settings.devEnabled}
                onChange={(checked) => onChange({ devEnabled: checked })}
                description={t(locale, "Visible by default in dev builds; gated in release.")}
              />
            </Surface>
            {settings.devEnabled ? (
              <DevPanel
                locale={locale}
                runtime={runtime}
                scenario={settings.scenario}
                onScenario={onScenario}
                onProbe={onProbe}
                onCalibrate={onCalibrate}
                onReload={onReload}
                onClearCache={onClearCache}
                onCopyDiagnostics={onCopyDiagnostics}
                onReset={onReset}
              />
            ) : (
              <Surface style={{ padding: 18 }}>
                <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 20 }}>
                  {t(
                    locale,
                    "Enable the master switch to show scenario presets, strict mode, and runtime health.",
                  )}
                </text>
              </Surface>
            )}
          </div>
        ) : null}

        {tab === "about" ? (
          <Surface style={{ padding: 22, display: "flex", flexDirection: "column", gap: 16 }}>
            <SectionHeader
              title="Live Linguist PC"
              subtitle={`${t(locale, "Version")} ${APP_VERSION} · ${t(locale, "Built for Windows 11 x64")}`}
            />
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              <InfoRow label={t(locale, "Version")} value={APP_VERSION} />
              <InfoRow label="UI" value="GPUIX 0.11.0 · React 19.2.3" />
              <InfoRow label={t(locale, "Backend")} value="llama.cpp · sherpa-onnx" />
              <InfoRow label={t(locale, "Licenses")} value="Apache-2.0 · MIT · product data" />
            </div>
            <Divider />
            <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
              {t(
                locale,
                "No telemetry. No cloud inference. Model downloads contact Hugging Face only when you start one.",
              )}
            </text>
            <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
              <Button label={t(locale, "Licenses")} kind="secondary" onClick={onLicenses} />
              <Button
                label={t(locale, "Copy diagnostics")}
                kind="ghost"
                onClick={onCopyDiagnostics}
              />
            </div>
          </Surface>
        ) : null}
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 16 }}>
      <text style={{ ...textStyle(type.meta, colors.faint), width: 120 }}>{label}</text>
      <text style={{ ...textStyle(type.body, colors.text), flexGrow: 1, minWidth: 0 }}>
        {value}
      </text>
    </div>
  );
}

function ModelCard({
  name,
  detail,
  state,
  ready,
}: {
  name: string;
  detail: string;
  state: string;
  ready?: boolean;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 12,
        padding: 14,
        borderRadius: 10,
        backgroundColor: "#FFFFFF06",
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(type.body, colors.text, 600)}>{name}</text>
        <text style={textStyle(type.meta, colors.faint)}>{detail}</text>
      </div>
      <text style={textStyle(type.meta, ready ? colors.mint : colors.muted, 600)}>{state}</text>
    </div>
  );
}
