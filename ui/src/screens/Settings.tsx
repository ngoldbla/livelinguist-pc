import { useState } from "react";
import {
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
import { languageByCode, languageByUiCode } from "../i18n/languages";
import { deviceLocales, resolveLocale, t } from "../i18n/locales";
import { CAPTION_SIZE_MAX, CAPTION_SIZE_MIN, type UiSettings } from "../settings";
import { allReady, type ModelView, percent } from "../store/modelStore";
import { LicensesContent } from "./Licenses";

/**
 * Mac settings shape: Captions, Models, Licenses, plus the PC Developer tab.
 * Captions is a form, not a dashboard; models downloads live in one card.
 */
export function SettingsScreen({
  locale,
  language,
  settings,
  onChange,
  runtime,
  onBack,
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
  language: string;
  settings: UiSettings;
  onChange: (patch: Partial<UiSettings>) => void;
  runtime: RuntimeStatus | null;
  onBack: () => void;
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
  const deviceUi = resolveLocale(false, null, deviceLocales());
  const learningName = languageByCode(language).shortName;
  const deviceName = languageByUiCode(deviceUi).shortName;

  const modelState = (id: string) => {
    const model = models?.models.find((entry) => entry.id === id);
    if (!model) return t(locale, "Not downloaded");
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
          { id: "licenses", label: t(locale, "Licenses") },
          { id: "developer", label: t(locale, "Developer") },
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
          <div style={{ display: "flex", justifyContent: "flex-start" }}>
            <Surface
              style={{
                width: "100%",
                maxWidth: 640,
                padding: 22,
                display: "flex",
                flexDirection: "column",
                gap: 18,
              }}
            >
              <SectionHeader
                title={t(locale, "Captions")}
                subtitle={t(locale, "Caption size and the two-tier display.")}
              />
              <Toggle
                label={t(locale, "Show live verbatim text (Tier 1)")}
                checked={settings.showTier1}
                onChange={(checked) => onChange({ showTier1: checked })}
                description={t(locale, "Two caption tiers: verbatim partial + simplified final.")}
              />
              <Slider
                label={t(locale, "Caption size")}
                value={settings.captionSize}
                min={CAPTION_SIZE_MIN}
                max={CAPTION_SIZE_MAX}
                unit="px"
                onChange={(value) => onChange({ captionSize: value })}
              />
              <Divider />
              <Toggle
                label={t(locale, "Detect language")}
                checked={settings.detection}
                onChange={(checked) => onChange({ detection: checked })}
                description={t(
                  locale,
                  "Language detection powers the English-reversion indicator.",
                )}
              />
              <text style={{ ...textStyle(type.meta, colors.faint), lineHeight: 18 }}>
                {t(locale, "Simplified captions always stay visible — they are the point.")}
              </text>
              <Divider />
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <text style={textStyle(type.body, colors.text, 600)}>
                  {t(locale, "App language")}
                </text>
                <Tabs
                  tabs={[
                    { id: "learning", label: learningName },
                    { id: "device", label: deviceName },
                  ]}
                  value={settings.uiLanguage}
                  onChange={(id) =>
                    onChange({ uiLanguage: id === "device" ? "device" : "learning" })
                  }
                />
                <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
                  {t(locale, "The app follows the language you're learning. Switch back any time.")}
                </text>
              </div>
            </Surface>
          </div>
        ) : null}

        {tab === "models" ? (
          <div style={{ display: "flex", justifyContent: "flex-start" }}>
            <Surface
              style={{
                width: "100%",
                maxWidth: 640,
                padding: 22,
                display: "flex",
                flexDirection: "column",
                gap: 14,
              }}
            >
              <SectionHeader
                title={t(locale, "On-device models")}
                subtitle={t(locale, "No models are bundled. Downloads are public and tokenless.")}
              />
              <ModelCard
                name="Qwen3 0.6B EasyLanguage"
                detail="GGUF Q4_K_M · Apache-2.0 · ~397 MB"
                state={modelState("llm-qwen3-0.6b-easylanguage")}
                ready={models?.models.some(
                  (entry) => entry.id === "llm-qwen3-0.6b-easylanguage" && entry.phase === "ready",
                )}
              />
              <ModelCard
                name="Nemotron 3.5 ASR 0.6B"
                detail="560 ms · int8 · 40 locales · ~682 MB"
                state={modelState("asr-nemotron-3.5-0.6b-560ms")}
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
                <Button
                  label={t(locale, "Licenses")}
                  kind="secondary"
                  onClick={() => setTab("licenses")}
                />
              </div>
              <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
                {t(
                  locale,
                  "Model downloads contact Hugging Face. Audio and transcripts stay on this device. Nothing is uploaded.",
                )}
              </text>
            </Surface>
          </div>
        ) : null}

        {tab === "licenses" ? <LicensesContent locale={locale} /> : null}

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
      </div>
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
