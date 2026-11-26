import { Select, SelectContent, SelectItem, SelectTrigger } from "@gpuix/react";
import { useState } from "react";
import { BrandMark, Button, Icon } from "../aurora/components";
import { colors, focusRing, textStyle, type } from "../aurora/theme";
import { LANGUAGES, languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import { planSession } from "../sessionPlan";
import { allReady, type ModelView } from "../store/modelStore";
import type { AudioSource } from "../store/sessionStore";

/**
 * Mac/iOS console: brand lockup, a state headline, two decision tiles with
 * native Select menus (language, sound), then one session cluster at the
 * bottom. The transcript lives on the live screen.
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
  const [openMenu, setOpenMenu] = useState<"language" | "sound" | null>(null);
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

  const languageItems = LANGUAGES.map((entry) => ({
    value: entry.code,
    label: entry.name,
    textValue: entry.name,
  }));
  const soundItems = [
    {
      value: "monitor",
      label: t(locale, "System audio"),
      textValue: t(locale, "System audio"),
    },
    { value: "mic", label: t(locale, "Microphone"), textValue: t(locale, "Microphone") },
    {
      value: "demo",
      label: t(locale, "Demo conversation"),
      textValue: t(locale, "Demo conversation"),
    },
  ];

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
              <Button
                label={t(locale, "Summary")}
                icon="captions"
                kind="ghost"
                onClick={onSummary}
              />
            ) : null}
            <Button
              label={t(locale, "Settings")}
              icon="settings"
              kind="secondary"
              onClick={onSettings}
            />
          </div>

          <text style={{ ...textStyle(40, colors.text, 700), lineHeight: 46 }}>
            {t(locale, "Ready to caption")}
          </text>

          <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
            <div style={{ flexGrow: 1, flexBasis: 0, minWidth: 0 }}>
              <Select
                items={languageItems}
                value={language}
                open={openMenu === "language"}
                onOpenChange={(open) => setOpenMenu(open ? "language" : null)}
                onValueChange={(code) => onLanguage(code)}
              >
                <SelectTrigger
                  role="button"
                  aria-haspopup="listbox"
                  aria-expanded={openMenu === "language"}
                  aria-label={`${t(locale, "LANGUAGE")}: ${selected.name}`}
                  style={(state) => tileStyle(state.open)}
                >
                  <TileBody
                    icon="globe"
                    label={t(locale, "LANGUAGE")}
                    value={selected.name}
                    caption={statusLabel(selected.asrStatus)}
                  />
                </SelectTrigger>
                <SelectContent
                  role="listbox"
                  aria-label={t(locale, "LANGUAGE")}
                  side="bottom"
                  align="start"
                  sideOffset={6}
                  style={menuStyle}
                >
                  {LANGUAGES.map((entry) => (
                    <SelectItem
                      key={entry.code}
                      value={entry.code}
                      role="option"
                      aria-selected={entry.code === language}
                      style={itemStyle}
                    >
                      <div
                        style={{
                          display: "flex",
                          flexDirection: "row",
                          alignItems: "center",
                          gap: 8,
                        }}
                      >
                        <text style={textStyle(type.body, colors.text, 500)}>{entry.name}</text>
                        <div style={{ flexGrow: 1 }} />
                        {entry.code === language ? (
                          <Icon name="check" size={14} color={colors.accent} />
                        ) : null}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div style={{ flexGrow: 1, flexBasis: 0, minWidth: 0 }}>
              <Select
                items={soundItems}
                value={source}
                open={openMenu === "sound"}
                onOpenChange={(open) => setOpenMenu(open ? "sound" : null)}
                onValueChange={(value) => {
                  if (value === "mic") {
                    onBlocked(micMessage);
                    return;
                  }
                  onSource(value as AudioSource);
                }}
              >
                <SelectTrigger
                  role="button"
                  aria-haspopup="listbox"
                  aria-expanded={openMenu === "sound"}
                  aria-label={`${t(locale, "SOUND")}: ${sourceName}`}
                  style={(state) => tileStyle(state.open)}
                >
                  <TileBody
                    icon="audio-waveform"
                    label={t(locale, "SOUND")}
                    value={sourceName}
                    caption={sourceCaption}
                  />
                </SelectTrigger>
                <SelectContent
                  role="listbox"
                  aria-label={t(locale, "SOUND")}
                  side="bottom"
                  align="start"
                  sideOffset={6}
                  style={menuStyle}
                >
                  <SelectItem
                    value="monitor"
                    role="option"
                    aria-selected={source === "monitor"}
                    style={itemStyle}
                  >
                    <ChoiceBody
                      icon="audio-waveform"
                      title={t(locale, "System audio")}
                      detail={t(locale, "Everything your computer plays")}
                      selected={source === "monitor"}
                    />
                  </SelectItem>
                  <SelectItem
                    value="mic"
                    role="option"
                    aria-selected={source === "mic"}
                    disabled
                    style={itemStyle}
                  >
                    <ChoiceBody
                      icon="mic"
                      title={t(locale, "Microphone")}
                      detail={micMessage}
                      selected={source === "mic"}
                    />
                  </SelectItem>
                  <SelectItem
                    value="demo"
                    role="option"
                    aria-selected={source === "demo"}
                    style={itemStyle}
                  >
                    <ChoiceBody
                      icon="captions"
                      title={t(locale, "Demo conversation")}
                      detail={t(locale, "Built-in French conversation")}
                      selected={source === "demo"}
                    />
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          {ready ? null : (
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
              <Icon name="info" size={14} color={colors.amber} />
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
          icon="audio-waveform"
          onClick={onStart}
          testId="start-captions"
          style={{ height: 48, minWidth: 240, paddingLeft: 24, paddingRight: 24 }}
        />
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 6 }}>
          <Icon name="lock" size={13} color={colors.faint} />
          <text style={textStyle(type.meta, colors.faint)}>
            {t(locale, "Offline — audio never leaves this device")}
          </text>
        </div>
      </div>
    </div>
  );
}

function tileStyle(open: boolean) {
  return {
    display: "flex",
    flexDirection: "column" as const,
    gap: 6,
    padding: 16,
    minHeight: 108,
    width: "100%",
    borderRadius: 12,
    backgroundColor: open ? "#AFA2FF14" : colors.surface,
    borderWidth: 1,
    borderColor: open ? "#AFA2FF66" : colors.border,
    cursor: "pointer" as const,
    hover: { backgroundColor: open ? "#AFA2FF1F" : "#FFFFFF0A" },
    focusVisible: focusRing,
  };
}

const menuStyle = {
  width: 320,
  maxHeight: 320,
  overflowY: "scroll" as const,
  padding: 6,
  borderRadius: 10,
  borderWidth: 1,
  borderColor: colors.borderStrong,
  backgroundColor: colors.surfaceRaised,
  boxShadow: {
    offsetX: 0,
    offsetY: 12,
    blurRadius: 24,
    spreadRadius: 0,
    color: "#00000088",
  },
  scrollbarColor: "#FFFFFF22",
  scrollbarWidth: 6,
};

function itemStyle(state: { selected: boolean; highlighted: boolean; disabled: boolean }) {
  return {
    display: "flex",
    flexDirection: "row" as const,
    alignItems: "center",
    gap: 8,
    paddingTop: 8,
    paddingBottom: 8,
    paddingLeft: 10,
    paddingRight: 10,
    borderRadius: 6,
    backgroundColor: state.selected
      ? colors.accentSoft
      : state.highlighted
        ? "#FFFFFF0A"
        : "#00000000",
    opacity: state.disabled ? 0.5 : 1,
    cursor: state.disabled ? ("not-allowed" as const) : ("pointer" as const),
  };
}

function TileBody({
  icon,
  label,
  value,
  caption,
}: {
  icon: "globe" | "audio-waveform";
  label: string;
  value: string;
  caption: string;
}) {
  return (
    <>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 6 }}>
        <Icon name={icon} size={13} color={colors.faint} />
        <text style={{ ...textStyle(type.meta, colors.faint, 700) }}>{label}</text>
        <div style={{ flexGrow: 1 }} />
        <Icon name="chevron-down" size={14} color={colors.faint} />
      </div>
      <div style={{ flexGrow: 1 }} />
      <text style={{ ...textStyle(type.title, colors.text, 600), lineHeight: 24 }}>{value}</text>
      <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 17 }}>{caption}</text>
    </>
  );
}

function ChoiceBody({
  icon,
  title,
  detail,
  selected,
}: {
  icon: "audio-waveform" | "mic" | "captions";
  title: string;
  detail: string;
  selected: boolean;
}) {
  return (
    <>
      <Icon name={icon} size={15} color={selected ? colors.accent : colors.muted} />
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 1,
          flexGrow: 1,
          minWidth: 0,
        }}
      >
        <text style={textStyle(type.body, colors.text, 600)}>{title}</text>
        <text style={textStyle(type.meta, colors.muted)}>{detail}</text>
      </div>
      {selected ? <Icon name="check" size={14} color={colors.accent} /> : null}
    </>
  );
}
