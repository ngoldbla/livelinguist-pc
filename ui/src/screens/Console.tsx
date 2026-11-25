import { useState } from "react";
import { BrandMark, Button, StatusDot, Surface } from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, focusRing, textStyle, type } from "../aurora/theme";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import { planSession } from "../sessionPlan";
import { allReady, type ModelView } from "../store/modelStore";
import type { AudioSource } from "../store/sessionStore";

type Panel = "language" | "sound" | null;

/**
 * Mac console flow: brand lockup, a state headline, two decision tiles
 * (language, sound), then one session cluster at the bottom. Choices open in
 * place; the transcript lives on the live screen.
 */
export function ConsoleScreen({
  language,
  locale,
  source,
  models,
  hasTranscript,
  onLanguage,
  onSource,
  onStart,
  onBlocked,
  onSettings,
  onSummary,
}: {
  language: string;
  locale: string;
  source: AudioSource;
  models: ModelView;
  hasTranscript: boolean;
  onLanguage: (code: string) => void;
  onSource: (source: AudioSource) => void;
  onStart: () => void;
  onBlocked: (message: string) => void;
  onSettings: () => void;
  onSummary: () => void;
}) {
  const selected = languageByCode(language);
  const [panel, setPanel] = useState<Panel>(null);
  const micPlan = planSession("mic");
  const micMessage = micPlan.ok ? "" : micPlan.error;
  const ready = allReady(models);

  const sourceName =
    source === "demo"
      ? t(locale, "Demo conversation")
      : source === "mic"
        ? t(locale, "Microphone")
        : t(locale, "System audio");
  const sourceCaption =
    source === "demo"
      ? t(locale, "Built-in French conversation")
      : source === "mic"
        ? micMessage
        : t(locale, "Everything your computer plays");

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div
        style={{
          flexGrow: 1,
          minHeight: 0,
          overflow: "scroll",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        <div
          style={{
            width: "100%",
            maxWidth: 640,
            display: "flex",
            flexDirection: "column",
            gap: 20,
            paddingLeft: 20,
            paddingRight: 20,
            paddingTop: 36,
            paddingBottom: 24,
          }}
        >
          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
            <BrandMark size={30} />
            <text style={{ ...textStyle(type.meta, colors.faint, 700) }}>
              {t(locale, "LIVE LINGUIST · ON-DEVICE")}
            </text>
            <div style={{ flexGrow: 1 }} />
            {hasTranscript ? (
              <Button label={t(locale, "Summary")} kind="ghost" onClick={onSummary} />
            ) : null}
            <Button label={t(locale, "Settings")} kind="secondary" onClick={onSettings} />
          </div>

          <text style={{ ...textStyle(40, colors.text, 700), lineHeight: 46 }}>
            {t(locale, "Ready to caption")}
          </text>

          <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
            <DecisionTile
              label={t(locale, "LANGUAGE")}
              value={selected.name}
              caption={statusLabel(selected.asrStatus)}
              open={panel === "language"}
              onClick={() => setPanel(panel === "language" ? null : "language")}
            />
            <DecisionTile
              label={t(locale, "SOUND")}
              value={sourceName}
              caption={sourceCaption}
              open={panel === "sound"}
              onClick={() => setPanel(panel === "sound" ? null : "sound")}
            />
          </div>

          {panel === "language" ? (
            <Surface
              style={{
                padding: 16,
                display: "flex",
                flexDirection: "column",
                gap: 12,
              }}
            >
              <LanguagePicker
                selected={language}
                onSelect={(code) => {
                  onLanguage(code);
                  setPanel(null);
                }}
                locale={locale}
                height={300}
                showHeader={false}
              />
              <div style={{ display: "flex", flexDirection: "row", justifyContent: "flex-end" }}>
                <Button label={t(locale, "Done")} kind="ghost" onClick={() => setPanel(null)} />
              </div>
            </Surface>
          ) : null}

          {panel === "sound" ? (
            <Surface style={{ padding: 8, display: "flex", flexDirection: "column", gap: 2 }}>
              <ChoiceRow
                title={t(locale, "System audio")}
                detail={t(locale, "Everything your computer plays")}
                selected={source === "monitor"}
                onClick={() => {
                  onSource("monitor");
                  setPanel(null);
                }}
              />
              <ChoiceRow
                title={t(locale, "Microphone")}
                detail={micMessage}
                selected={source === "mic"}
                disabled
                onClick={() => onBlocked(micMessage)}
              />
              <ChoiceRow
                title={t(locale, "Demo conversation")}
                detail={t(locale, "Built-in French conversation")}
                selected={source === "demo"}
                onClick={() => {
                  onSource("demo");
                  setPanel(null);
                }}
              />
            </Surface>
          ) : null}

          {ready ? null : (
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
              <StatusDot tone="amber" label="" />
              <text style={textStyle(type.meta, colors.muted)}>
                {t(locale, "Model download pending — captions run in demo mode")}
              </text>
            </div>
          )}
        </div>
      </div>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 10,
          paddingTop: 14,
          paddingBottom: 20,
        }}
      >
        <Button
          label={t(locale, "Start captions")}
          onClick={onStart}
          testId="start-captions"
          style={{ height: 48, minWidth: 240, paddingLeft: 24, paddingRight: 24 }}
        />
        <text style={textStyle(type.meta, colors.faint)}>
          {t(locale, "Offline — audio never leaves this device")}
        </text>
      </div>
    </div>
  );
}

function DecisionTile({
  label,
  value,
  caption,
  open,
  onClick,
}: {
  label: string;
  value: string;
  caption: string;
  open: boolean;
  onClick: () => void;
}) {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`${label}: ${value}`}
      aria-description={caption}
      aria-expanded={open}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      style={{
        flexGrow: 1,
        flexShrink: 1,
        minWidth: 0,
        minHeight: 108,
        padding: 16,
        borderRadius: 12,
        display: "flex",
        flexDirection: "column",
        gap: 6,
        backgroundColor: open ? "#AFA2FF14" : colors.surface,
        borderWidth: 1,
        borderColor: open ? "#AFA2FF66" : colors.border,
        cursor: "pointer",
        hover: { backgroundColor: open ? "#AFA2FF1F" : "#FFFFFF0A" },
        focusVisible: focusRing,
      }}
    >
      <text style={{ ...textStyle(type.meta, colors.faint, 700) }}>{label}</text>
      <div style={{ flexGrow: 1 }} />
      <text style={{ ...textStyle(type.title, colors.text, 600), lineHeight: 24 }}>{value}</text>
      <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 17 }}>{caption}</text>
    </div>
  );
}

function ChoiceRow({
  title,
  detail,
  selected,
  disabled = false,
  onClick,
}: {
  title: string;
  detail: string;
  selected: boolean;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={title}
      aria-selected={selected}
      aria-disabled={disabled || undefined}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 10,
        paddingTop: 10,
        paddingBottom: 10,
        paddingLeft: 10,
        paddingRight: 10,
        borderRadius: 8,
        backgroundColor: selected ? colors.accentSoft : "#00000000",
        opacity: disabled ? 0.55 : 1,
        cursor: disabled ? "not-allowed" : "pointer",
        hover: disabled ? undefined : { backgroundColor: "#FFFFFF0A" },
        focusVisible: focusRing,
      }}
    >
      <div
        style={{
          width: 14,
          height: 14,
          borderRadius: 7,
          borderWidth: 2,
          borderColor: selected ? colors.accent : colors.faint,
          backgroundColor: selected ? colors.accent : "#00000000",
          flexShrink: 0,
        }}
      />
      <div style={{ display: "flex", flexDirection: "column", gap: 1, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(type.body, colors.text, 600)}>{title}</text>
        <text style={textStyle(type.meta, colors.muted)}>{detail}</text>
      </div>
    </div>
  );
}
