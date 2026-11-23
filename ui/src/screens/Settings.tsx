import { useState } from "react";
import {
  Badge,
  Button,
  Divider,
  SectionHeader,
  Slider,
  Surface,
  Tabs,
  Toast,
  Toggle,
} from "../aurora/components";
import { colors, gradient, textStyle } from "../aurora/theme";
import type { RuntimeStatus } from "../bridge/generated/RuntimeStatus";
import { DevPanel } from "../dev/DevPanel";
import { t } from "../i18n/locales";

export interface UiSettings {
  captionSize: number;
  showTier1: boolean;
  detection: boolean;
  reducedEffects: boolean;
  followLearning: boolean;
  devEnabled: boolean;
  scenario: string;
}

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
}) {
  const [tab, setTab] = useState(process.env.LL_TAB ?? "captions");
  const [toast, setToast] = useState<string | null>(null);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 18, padding: 28, height: "100%" }}>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
        <Button label={t(locale, "Back")} kind="secondary" onClick={onBack} />
        <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1 }}>
          <text style={{ ...textStyle(10, colors.accent, 700) }}>
            {t(locale, "SETTINGS").toUpperCase()}
          </text>
          <text style={textStyle(24, colors.text, 700)}>{t(locale, "Settings")}</text>
        </div>
        <Badge label={t(locale, "Content-free diagnostics")} tone="mint" />
      </div>

      <Tabs
        tabs={[
          { id: "captions", label: t(locale, "CAPTIONS") },
          { id: "models", label: t(locale, "MODELS") },
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
          <div style={{ display: "flex", flexDirection: "row", gap: 16 }}>
            <Surface
              style={{
                flexGrow: 1,
                padding: 20,
                display: "flex",
                flexDirection: "column",
                gap: 16,
              }}
            >
              <SectionHeader
                title={t(locale, "Captions")}
                subtitle={t(locale, "Caption size and the two-tier display.")}
              />
              <Slider
                label={t(locale, "Caption size")}
                value={settings.captionSize}
                min={14}
                max={30}
                unit="px"
                onChange={(value) => onChange({ captionSize: value })}
              />
              <Divider />
              <Toggle
                label={t(locale, "Show live verbatim text (Tier 1)")}
                checked={settings.showTier1}
                onChange={(checked) => onChange({ showTier1: checked })}
                description="Verbatim partials above the simplified final."
              />
              <Toggle
                label={t(locale, "Detect language")}
                checked={settings.detection}
                onChange={(checked) => onChange({ detection: checked })}
                description="Powers the English-reversion indicator."
              />
              <Toggle
                label={t(locale, "Reduced effects")}
                checked={settings.reducedEffects}
                onChange={(checked) => onChange({ reducedEffects: checked })}
                description="Static field, no motion, 30 fps target on low-end machines."
              />
              <Divider />
              <Toggle
                label={t(locale, "App language follows learning language")}
                checked={settings.followLearning}
                onChange={(checked) => onChange({ followLearning: checked })}
                description="Switch back to the device language any time."
              />
              <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
                <text style={textStyle(12, colors.muted)}>{t(locale, "App language")}</text>
                <div style={{ flexGrow: 1 }} />
                <Badge label={settings.followLearning ? locale : "en"} tone="accent" />
              </div>
            </Surface>
            <Surface
              style={{
                width: 300,
                flexShrink: 0,
                padding: 18,
                display: "flex",
                flexDirection: "column",
                gap: 10,
              }}
            >
              <text style={{ ...textStyle(10, colors.faint, 700) }}>
                {t(locale, "PREVIEW").toUpperCase()}
              </text>
              <div
                style={{
                  ...{ padding: 10 },
                  backgroundColor: "#00000033",
                  borderRadius: 10,
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                }}
              >
                <text style={textStyle(11, colors.muted)}>
                  On s'est retrouvés coincés dans les embouteillages pendant une éternité.
                </text>
                <text style={textStyle(settings.captionSize * 0.8, "#E4DFFF", 600)}>
                  Il y avait beaucoup de voitures. On a attendu très longtemps.
                </text>
              </div>
              <text style={textStyle(11, colors.faint)}>
                {t(
                  locale,
                  "Validators are confidence signals, not gates. Flagged captions stay visible.",
                )}
              </text>
            </Surface>
          </div>
        ) : null}

        {tab === "models" ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            <Surface style={{ padding: 20, display: "flex", flexDirection: "column", gap: 12 }}>
              <SectionHeader
                title={t(locale, "On-device models")}
                subtitle={t(locale, "No models are bundled. Downloads are public and tokenless.")}
              />
              <ModelCard
                name="Qwen3 0.6B EasyLanguage"
                detail="GGUF Q4_K_M · Apache-2.0"
                size="~397 MB"
                state="pinned · first run"
                tone="accent"
              />
              <ModelCard
                name="Nemotron 3.5 ASR 0.6B"
                detail="560 ms · int8 · 40 locales"
                size="~682 MB"
                state="pinned · first run"
                tone="mint"
              />
              <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
                <Button
                  label={t(locale, "Download models")}
                  onClick={() =>
                    setToast(
                      "Downloads are not wired in this preview build. Both model hashes are pinned in the catalog.",
                    )
                  }
                />
                <Button label={t(locale, "Licenses")} kind="secondary" onClick={onLicenses} />
              </div>
              {toast ? <Toast message={toast} tone="amber" /> : null}
            </Surface>
            <Surface style={{ padding: 16 }}>
              <text style={textStyle(11, colors.muted)}>
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
            <Surface
              style={{
                padding: 16,
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                gap: 12,
              }}
            >
              <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1 }}>
                <text style={textStyle(13, colors.text, 600)}>
                  {t(locale, "Developer settings")}
                </text>
                <text style={textStyle(11, colors.muted)}>
                  {t(locale, "Visible by default in dev builds; gated in release.")}
                </text>
              </div>
              <Toggle
                label=""
                checked={settings.devEnabled}
                onChange={(checked) => onChange({ devEnabled: checked })}
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
                <text style={textStyle(11, colors.muted)}>
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
          <Surface style={{ padding: 22, display: "flex", flexDirection: "column", gap: 14 }}>
            <SectionHeader
              title="Live Linguist PC"
              subtitle={`${t(locale, "Version")} 0.1.0 · ${t(locale, "Built for Windows 11 x64")}`}
            />
            <div style={{ display: "flex", flexDirection: "row", gap: 10, flexWrap: "wrap" }}>
              <Badge label="GPUIX 0.11.0" tone="accent" />
              <Badge label="React 19.2.3" tone="accent" />
              <Badge label="Rust" tone="mint" />
              <Badge label="llama.cpp" tone="neutral" />
              <Badge label="sherpa-onnx" tone="neutral" />
            </div>
            <Divider />
            <text style={textStyle(12, colors.muted)}>
              {t(locale, "Linux dev preview")} · {t(locale, "Built for Windows 11 x64")}.{" "}
              {t(locale, "Offline — audio never leaves this device")}.
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

function ModelCard({
  name,
  detail,
  size,
  state,
  tone,
}: {
  name: string;
  detail: string;
  size: string;
  state: string;
  tone: "accent" | "amber" | "mint";
}) {
  const color = tone === "accent" ? colors.accent : tone === "amber" ? colors.amber : colors.mint;
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 12,
        padding: 12,
        borderRadius: 10,
        backgroundColor: "#FFFFFF06",
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      <div
        style={{
          width: 34,
          height: 34,
          borderRadius: 10,
          background: gradient(`${color}44`, `${color}11`, 120),
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <text style={textStyle(10, color, 800)}>◇</text>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(13, colors.text, 600)}>{name}</text>
        <text style={textStyle(10, colors.faint)}>{detail}</text>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 2, alignItems: "flex-end" }}>
        <text style={textStyle(11, colors.muted)}>{size}</text>
        <text style={textStyle(9, color, 700)}>{state}</text>
      </div>
    </div>
  );
}
