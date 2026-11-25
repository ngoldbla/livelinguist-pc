import { useState } from "react";
import { BrandMark, Button, StatusDot, Surface, Tabs, Toggle } from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode, statusDetail, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import { planSession } from "../sessionPlan";
import type { UiSettings } from "../settings";
import { allReady, type ModelView } from "../store/modelStore";
import type { AudioSource, SimplificationMode } from "../store/sessionStore";

/**
 * The console is one decision after another, top to bottom: language, what to
 * listen to, start. Models, privacy and licenses live in the footer. Nothing
 * here is a control panel for its own sake.
 */
export function ConsoleScreen({
  language,
  locale,
  source,
  mode,
  models,
  settings,
  hasTranscript,
  onLanguage,
  onSource,
  onMode,
  onChange,
  onStart,
  onBlocked,
  onSettings,
  onLicenses,
  onSummary,
}: {
  language: string;
  locale: string;
  source: AudioSource;
  mode: SimplificationMode;
  models: ModelView;
  settings: UiSettings;
  hasTranscript: boolean;
  onLanguage: (code: string) => void;
  onSource: (source: AudioSource) => void;
  onMode: (mode: SimplificationMode) => void;
  onChange: (patch: Partial<UiSettings>) => void;
  onStart: () => void;
  onBlocked: (message: string) => void;
  onSettings: () => void;
  onLicenses: () => void;
  onSummary: () => void;
}) {
  const selected = languageByCode(language);
  const [showLanguages, setShowLanguages] = useState(false);
  const micPlan = planSession("mic");
  const micMessage = micPlan.ok ? "" : micPlan.error;
  const ready = allReady(models);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", position: "relative" }}>
      <div
        style={{
          flexGrow: 1,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          paddingTop: 24,
          paddingBottom: 76,
          paddingLeft: 28,
          paddingRight: 28,
        }}
      >
        <div
          style={{
            width: "100%",
            maxWidth: 760,
            display: "flex",
            flexDirection: "column",
            gap: 18,
          }}
        >
          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
            <BrandMark size={34} />
            <div style={{ display: "flex", flexDirection: "column", gap: 1, flexGrow: 1 }}>
              <text style={textStyle(type.label, colors.text, 600)}>Live Linguist</text>
              <text style={textStyle(type.meta, colors.faint)}>
                {t(locale, "Same language, not translation")}
              </text>
            </div>
            {hasTranscript ? (
              <Button label={t(locale, "Summary")} kind="ghost" onClick={onSummary} />
            ) : null}
            <Button label={t(locale, "Settings")} kind="secondary" onClick={onSettings} />
          </div>

          <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 14 }}>
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 14 }}>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 4,
                  flexGrow: 1,
                  minWidth: 0,
                }}
              >
                <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
                  {t(locale, "Learning language")}
                </text>
                <text style={{ ...textStyle(type.page, colors.text, 600), lineHeight: 34 }}>
                  {selected.name}
                </text>
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  <StatusDot
                    tone={selected.asrStatus === "ok" ? "mint" : "amber"}
                    label={statusLabel(selected.asrStatus)}
                  />
                  <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
                    {statusDetail(selected.asrStatus)}
                  </text>
                </div>
              </div>
              <Button
                label={showLanguages ? t(locale, "Done") : t(locale, "Change")}
                kind="secondary"
                onClick={() => setShowLanguages((current) => !current)}
              />
            </div>
            {showLanguages ? (
              <LanguagePicker
                selected={language}
                onSelect={(code) => {
                  onLanguage(code);
                  setShowLanguages(false);
                }}
                locale={locale}
                height={240}
                showHeader={false}
              />
            ) : null}
          </Surface>

          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
              {t(locale, "Sound").toUpperCase()}
            </text>
            <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
              <SourceTile
                title={t(locale, "Demo conversation")}
                detail={t(
                  locale,
                  "Demo mode shows the caption experience with a built-in conversation — no download or permissions needed. Live captioning of real audio needs the models.",
                )}
                active={source === "demo"}
                onClick={() => onSource("demo")}
              />
              <SourceTile
                title={t(locale, "System audio")}
                detail={t(locale, "WASAPI loopback on Windows; PipeWire monitor on Linux dev.")}
                active={source === "monitor"}
                onClick={() => onSource("monitor")}
              />
            </div>
            <SourceTile
              title={t(locale, "Microphone")}
              detail={micMessage}
              active={source === "mic"}
              disabled
              compact
              onClick={() => onBlocked(micMessage)}
            />
          </div>

          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 14 }}>
            <Button
              label={t(locale, "Start captions")}
              onClick={onStart}
              testId="start-captions"
              style={{ height: 44, paddingLeft: 22, paddingRight: 22 }}
            />
            <Tabs
              tabs={[
                { id: "on_demand", label: t(locale, "On-demand") },
                { id: "live", label: t(locale, "Live") },
              ]}
              value={mode}
              onChange={(id) => onMode(id as SimplificationMode)}
            />
            <text style={{ ...textStyle(type.meta, colors.faint), flexGrow: 1 }}>
              {mode === "on_demand"
                ? t(locale, "Pause any time and click a sentence to see it in simpler words.")
                : t(locale, "Simplified captions always stay visible — they are the point.")}
            </text>
          </div>

          <Surface
            style={{
              padding: 14,
              display: "flex",
              flexDirection: "row",
              alignItems: "center",
              gap: 24,
            }}
          >
            <div style={{ flexGrow: 1, minWidth: 0 }}>
              <Toggle
                label={t(locale, "Show live verbatim text (Tier 1)")}
                checked={settings.showTier1}
                onChange={(checked) => onChange({ showTier1: checked })}
              />
            </div>
            <div style={{ width: 1, alignSelf: "stretch", backgroundColor: colors.border }} />
            <div style={{ flexGrow: 1, minWidth: 0 }}>
              <Toggle
                label={t(locale, "Detect language")}
                checked={settings.detection}
                onChange={(checked) => onChange({ detection: checked })}
              />
            </div>
          </Surface>
        </div>
      </div>

      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          bottom: 0,
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: 14,
          paddingTop: 12,
          paddingBottom: 12,
          paddingLeft: 24,
          paddingRight: 24,
          borderTopWidth: 1,
          borderColor: colors.border,
          backgroundColor: colors.bg,
        }}
      >
        <StatusDot
          tone={ready ? "mint" : "amber"}
          label={ready ? t(locale, "Models ready") : t(locale, "Models not downloaded")}
        />
        <text style={textStyle(type.meta, colors.faint)}>
          {t(locale, "Offline — audio never leaves this device")}
        </text>
        <div style={{ flexGrow: 1 }} />
        <Button label={t(locale, "Licenses")} kind="ghost" onClick={onLicenses} />
      </div>
    </div>
  );
}

function SourceTile({
  title,
  detail,
  active,
  disabled = false,
  compact = false,
  onClick,
}: {
  title: string;
  detail: string;
  active: boolean;
  disabled?: boolean;
  compact?: boolean;
  onClick: () => void;
}) {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={title}
      aria-selected={active}
      aria-disabled={disabled || undefined}
      aria-description={detail}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      style={{
        display: "flex",
        flexDirection: compact ? "row" : "column",
        alignItems: compact ? "center" : "flex-start",
        gap: compact ? 10 : 8,
        padding: compact ? 12 : 14,
        flexGrow: 1,
        flexShrink: 1,
        minWidth: 0,
        borderRadius: 10,
        backgroundColor: active ? colors.accentSoft : "#FFFFFF05",
        borderWidth: 1,
        borderColor: active ? "#AFA2FF66" : colors.border,
        opacity: disabled ? 0.55 : 1,
        cursor: disabled ? "not-allowed" : "pointer",
        hover: disabled ? undefined : { backgroundColor: active ? "#AFA2FF2E" : "#FFFFFF0D" },
        focusVisible: {
          outlineWidth: 2,
          outlineColor: colors.accent,
          outlineOffset: 2,
        },
      }}
    >
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 9 }}>
        <div
          style={{
            width: 14,
            height: 14,
            borderRadius: 7,
            borderWidth: 2,
            borderColor: active ? colors.accent : colors.faint,
            backgroundColor: active ? colors.accent : "#00000000",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          {active ? (
            <div style={{ width: 5, height: 5, borderRadius: 3, backgroundColor: colors.bg }} />
          ) : null}
        </div>
        <text style={textStyle(type.body, colors.text, 600)}>{title}</text>
      </div>
      {disabled ? (
        <text style={{ ...textStyle(type.meta, colors.faint), flexGrow: 1, minWidth: 0 }}>
          {detail}
        </text>
      ) : (
        <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>{detail}</text>
      )}
    </div>
  );
}
