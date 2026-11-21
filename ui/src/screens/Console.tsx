import { useState } from "react";
import { Badge, Button, Divider, SectionHeader, Surface, Tabs, Toggle } from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, gradient, textStyle } from "../aurora/theme";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { AudioSource, SimplificationMode } from "../store/sessionStore";

export function ConsoleScreen({
  language,
  locale,
  source,
  mode,
  onLanguage,
  onSource,
  onMode,
  onStart,
  onSettings,
  onLicenses,
  onSummary,
}: {
  language: string;
  locale: string;
  source: AudioSource;
  mode: SimplificationMode;
  onLanguage: (code: string) => void;
  onSource: (source: AudioSource) => void;
  onMode: (mode: SimplificationMode) => void;
  onStart: () => void;
  onSettings: () => void;
  onLicenses: () => void;
  onSummary: () => void;
}) {
  const selected = languageByCode(language);
  const [showLanguages, setShowLanguages] = useState(false);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        gap: 20,
        padding: 24,
        height: "100%",
        minWidth: 0,
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 14,
          flexGrow: 1,
          flexShrink: 1,
          minWidth: 0,
        }}
      >
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
          <div
            style={{
              width: 38,
              height: 38,
              borderRadius: 12,
              background: gradient("#8B7CFF", "#2ED3B7", 135),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <text style={textStyle(14, "#0B0B12", 800)}>LL</text>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1 }}>
            <text style={textStyle(20, colors.text, 700)}>{t(locale, "CONSOLE")}</text>
            <text style={textStyle(11, colors.faint)}>
              {t(locale, "Same-language easy captions · on-device")}
            </text>
          </div>
          <Button label={t(locale, "Summary")} kind="ghost" onClick={onSummary} />
          <Button label={t(locale, "Settings")} kind="secondary" onClick={onSettings} />
        </div>

        <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 10 }}>
          <SectionHeader
            eyebrow={t(locale, "LEARNING LANGUAGE")}
            title={selected.name}
            subtitle={t(locale, "You're learning %@", selected.shortName)}
            trailing={
              <div style={{ display: "flex", flexDirection: "row", gap: 8, alignItems: "center" }}>
                <Badge
                  label={statusLabel(selected.asrStatus)}
                  tone={selected.asrStatus === "simplify_only" ? "amber" : "mint"}
                />
                <Button
                  label={showLanguages ? t(locale, "Done") : t(locale, "Change")}
                  kind="secondary"
                  onClick={() => setShowLanguages((current) => !current)}
                />
              </div>
            }
          />
          {showLanguages ? (
            <LanguagePicker
              selected={language}
              onSelect={(code) => {
                onLanguage(code);
                setShowLanguages(false);
              }}
              locale={locale}
              height={250}
              showHeader={false}
            />
          ) : (
            <div style={{ display: "flex", flexDirection: "row", gap: 10, flexWrap: "wrap" }}>
              <Badge label={selected.pack} tone="accent" />
              <Badge label={selected.code} tone="neutral" />
              <Badge
                label={
                  selected.asrStatus === "simplify_only"
                    ? "demo / typed input"
                    : "live ASR available"
                }
                tone={selected.asrStatus === "simplify_only" ? "amber" : "mint"}
              />
            </div>
          )}
          {selected.asrStatus === "simplify_only" ? (
            <text style={textStyle(12, colors.amber)}>
              {t(
                locale,
                "Live listening is not available for this language yet. Paste or type text to simplify it.",
              )}
            </text>
          ) : null}
        </Surface>

        <div style={{ display: "flex", flexDirection: "row", gap: 12, minWidth: 0 }}>
          <Surface
            style={{
              padding: 16,
              flexGrow: 1,
              flexShrink: 1,
              minWidth: 0,
              display: "flex",
              flexDirection: "column",
              gap: 12,
            }}
          >
            <text style={{ ...textStyle(10, colors.faint, 700) }}>
              {t(locale, "SOUND").toUpperCase()}
            </text>
            <SourceTile
              title={t(locale, "Demo conversation")}
              detail={t(
                locale,
                "Demo mode shows the caption experience with a built-in conversation — no download or permissions needed. Live captioning of real audio needs the models.",
              )}
              active={source === "demo"}
              badge={t(locale, "Ready")}
              onClick={() => onSource("demo")}
            />
            <SourceTile
              title={t(locale, "Microphone")}
              detail={t(
                locale,
                "Live listening is not available for this language yet. Paste or type text to simplify it.",
              )}
              active={source === "mic"}
              badge="pending"
              disabled
              onClick={() => onSource("mic")}
            />
            <SourceTile
              title={t(locale, "System audio")}
              detail={t(locale, "WASAPI loopback on Windows; PipeWire monitor on Linux dev.")}
              active={source === "monitor"}
              badge="pending"
              disabled
              onClick={() => onSource("monitor")}
            />
          </Surface>

          <Surface
            style={{
              padding: 16,
              width: 280,
              flexShrink: 0,
              display: "flex",
              flexDirection: "column",
              gap: 14,
            }}
          >
            <text style={{ ...textStyle(10, colors.faint, 700) }}>
              {t(locale, "CAPTIONING").toUpperCase()}
            </text>
            <Tabs
              tabs={[
                { id: "on_demand", label: t(locale, "On-demand") },
                { id: "live", label: t(locale, "Live") },
              ]}
              value={mode}
              onChange={(id) => onMode(id as SimplificationMode)}
            />
            <text style={textStyle(12, colors.muted)}>
              {mode === "on_demand"
                ? t(locale, "Pause any time and click a sentence to see it in simpler words.")
                : t(locale, "Simplified captions always stay visible — they are the point.")}
            </text>
            <Divider />
            <Toggle
              label={t(locale, "Show live verbatim text (Tier 1)")}
              checked
              onChange={() => undefined}
              description={t(locale, "Two caption tiers: verbatim partial + simplified final.")}
            />
            <Toggle
              label={t(locale, "Detect language")}
              checked
              onChange={() => undefined}
              description={t(locale, "Language detection powers the English-reversion indicator.")}
            />
            <div style={{ flexGrow: 1 }} />
            <Button label={t(locale, "Start captions")} onClick={onStart} testId="start-captions" />
            <div
              style={{ display: "flex", flexDirection: "row", gap: 8, justifyContent: "center" }}
            >
              <text style={textStyle(10, colors.faint)}>
                {t(locale, "Offline — audio never leaves this device")}
              </text>
            </div>
          </Surface>
        </div>

        <Surface
          style={{
            padding: 12,
            display: "flex",
            flexDirection: "row",
            alignItems: "center",
            gap: 12,
          }}
        >
          <Badge label={t(locale, "DEMO READY")} tone="mint" />
          <text style={{ ...textStyle(12, colors.muted), flexGrow: 1 }}>
            {t(locale, "Ready. Pick a sound source to start live, simplified captions.")}
          </text>
          <div
            role="button"
            tabIndex={0}
            aria-label={t(locale, "Licenses")}
            onClick={onLicenses}
            style={{ cursor: "pointer" }}
          >
            <text style={textStyle(11, colors.faint)}>{t(locale, "Licenses")} →</text>
          </div>
        </Surface>
      </div>

      <div style={{ width: 300, flexShrink: 0, display: "flex", flexDirection: "column", gap: 16 }}>
        <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 12 }}>
          <text style={{ ...textStyle(10, colors.faint, 700) }}>
            {t(locale, "ON-DEVICE MODELS").toUpperCase()}
          </text>
          <ModelStatus
            name="Qwen3 0.6B"
            detail="EasyLanguage · Q4_K_M"
            status={t(locale, "Demo backend")}
            tone="accent"
          />
          <ModelStatus
            name="Nemotron ASR"
            detail="streaming · int8"
            status={t(locale, "Not downloaded")}
            tone="amber"
          />
          <ModelStatus
            name="Silero VAD"
            detail="voice activity"
            status={t(locale, "Bundled logic")}
            tone="mint"
          />
          <Button label={t(locale, "Download models")} kind="secondary" onClick={() => undefined} />
        </Surface>

        <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 10 }}>
          <text style={{ ...textStyle(10, colors.faint, 700) }}>
            {t(locale, "SESSION").toUpperCase()}
          </text>
          <text style={textStyle(12, colors.muted)}>
            {t(
              locale,
              "Everything this device hears is processed locally. Nothing leaves the device.",
            )}
          </text>
          <Divider />
          <text style={textStyle(11, colors.faint)}>
            {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
          </text>
          <div
            role="button"
            tabIndex={0}
            aria-label={t(locale, "Settings")}
            onClick={onSettings}
            style={{ cursor: "pointer", paddingTop: 4 }}
          >
            <text style={textStyle(11, colors.muted)}>{t(locale, "Settings")} →</text>
          </div>
        </Surface>
      </div>
    </div>
  );
}

function SourceTile({
  title,
  detail,
  active,
  badge,
  disabled = false,
  onClick,
}: {
  title: string;
  detail: string;
  active: boolean;
  badge: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <div
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-label={title}
      aria-selected={active}
      onClick={() => {
        if (!disabled) onClick();
      }}
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 10,
        padding: 10,
        borderRadius: 10,
        backgroundColor: active ? colors.accentSoft : "#FFFFFF05",
        borderWidth: 1,
        borderColor: active ? "#8B7CFF55" : colors.border,
        opacity: disabled ? 0.55 : 1,
        cursor: disabled ? "not-allowed" : "pointer",
        hover: disabled ? undefined : { backgroundColor: active ? "#8B7CFF30" : "#FFFFFF0D" },
      }}
    >
      <div
        style={{
          width: 8,
          height: 8,
          borderRadius: 4,
          backgroundColor: active ? colors.mint : colors.faint,
        }}
      />
      <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(13, colors.text, 600)}>{title}</text>
        <text style={textStyle(10, colors.muted)}>{detail}</text>
      </div>
      <Badge label={badge} tone={disabled ? "neutral" : "mint"} />
    </div>
  );
}

function ModelStatus({
  name,
  detail,
  status,
  tone,
}: {
  name: string;
  detail: string;
  status: string;
  tone: "accent" | "amber" | "mint";
}) {
  const color = tone === "accent" ? colors.accent : tone === "amber" ? colors.amber : colors.mint;
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 9 }}>
      <div style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: color }} />
      <div style={{ display: "flex", flexDirection: "column", gap: 1, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(12, colors.text, 600)}>{name}</text>
        <text style={textStyle(10, colors.faint)}>{detail}</text>
      </div>
      <text style={textStyle(10, color, 600)}>{status}</text>
    </div>
  );
}
