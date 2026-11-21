import { useState } from "react";
import {
  Badge,
  Button,
  CaptionRow,
  Divider,
  SectionHeader,
  SummaryTile,
  Surface,
  Toast,
} from "../aurora/components";
import { colors, textStyle } from "../aurora/theme";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

export function SummaryScreen({
  state,
  locale,
  onExport,
  onBack,
  onConsole,
}: {
  state: SessionView;
  locale: string;
  onExport: (format: "markdown" | "plain" | "csv") => Promise<void>;
  onBack: () => void;
  onConsole: () => void;
}) {
  const [toast, setToast] = useState<string | null>(null);
  const duration = state.summary?.durationS ?? 0;
  const minutes = Math.floor(duration / 60);
  const seconds = Math.floor(duration % 60);
  const detected = state.segments.filter((segment) => segment.detectedLanguage);
  const base = state.language.split("-")[0];
  const targetPercent =
    detected.length === 0
      ? null
      : Math.round(
          (detected.filter((segment) => segment.detectedLanguage === base).length /
            detected.length) *
            100,
        );
  const simplified = state.segments.filter((segment) => segment.simplified).length;

  const exportAs = async (format: "markdown" | "plain" | "csv") => {
    try {
      await onExport(format);
      setToast(`${t(locale, "Export transcript")} · ${format}`);
    } catch {
      setToast("Export failed. Check the destination path.");
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 18, padding: 28, height: "100%" }}>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
        <Button label={t(locale, "Back")} kind="secondary" onClick={onBack} />
        <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1 }}>
          <text style={{ ...textStyle(10, colors.accent, 700) }}>
            {t(locale, "SESSION SUMMARY").toUpperCase()}
          </text>
          <text style={textStyle(24, colors.text, 700)}>{t(locale, "Session summary")}</text>
        </div>
        <Badge
          label={state.phase === "summary" ? t(locale, "Done") : t(locale, "Listening")}
          tone={state.phase === "summary" ? "mint" : "amber"}
        />
      </div>

      <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
        <SummaryTile
          label={t(locale, "DURATION")}
          value={`${minutes} min ${seconds} s`}
          hint={`${Math.round(duration)} seconds`}
        />
        <SummaryTile
          label={t(locale, "SENTENCES")}
          value={String(state.segments.length)}
          hint={`${simplified} simplified`}
        />
        <SummaryTile
          label={t(locale, "IN TARGET LANGUAGE")}
          value={targetPercent == null ? "—" : `${targetPercent}%`}
          hint={state.language}
          tone={colors.mint}
        />
        <SummaryTile
          label={t(locale, "ENGLISH REVERSIONS")}
          value={String(state.reversionActive ? 1 : 0)}
          hint={t(locale, "detected segments")}
        />
      </div>

      <div style={{ display: "flex", flexDirection: "row", gap: 16, flexGrow: 1, minHeight: 0 }}>
        <Surface
          style={{
            flexGrow: 1,
            padding: 16,
            display: "flex",
            flexDirection: "column",
            gap: 10,
            minHeight: 0,
          }}
        >
          <SectionHeader
            title={t(locale, "Transcript")}
            subtitle={t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
          />
          <div
            style={{
              flexGrow: 1,
              minHeight: 0,
              overflow: "scroll",
              scrollbarColor: "#FFFFFF22",
              scrollbarWidth: 6,
              display: "flex",
              flexDirection: "column",
              gap: 8,
            }}
          >
            {state.segments.length === 0 ? (
              <text style={textStyle(12, colors.muted)}>{t(locale, "No session data.")}</text>
            ) : (
              state.segments.map((segment) => (
                <CaptionRow key={segment.id} segment={segment} locale={locale} compact />
              ))
            )}
          </div>
        </Surface>

        <Surface
          style={{
            width: 320,
            flexShrink: 0,
            padding: 18,
            display: "flex",
            flexDirection: "column",
            gap: 12,
          }}
        >
          <text style={{ ...textStyle(10, colors.faint, 700) }}>
            {t(locale, "EXPORT").toUpperCase()}
          </text>
          <Button label={t(locale, "Markdown")} onClick={() => void exportAs("markdown")} />
          <Button
            label={t(locale, "Plain text (.txt)")}
            kind="secondary"
            onClick={() => void exportAs("plain")}
          />
          <Button
            label={t(locale, "Diagnostics CSV")}
            kind="secondary"
            onClick={() => void exportAs("csv")}
          />
          <Divider />
          <text style={textStyle(11, colors.muted)}>
            {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
          </text>
          <Badge label={t(locale, "Content-free diagnostics")} tone="mint" />
          <div style={{ flexGrow: 1 }} />
          <Button label={t(locale, "Back to console")} kind="ghost" onClick={onConsole} />
          {toast ? <Toast message={toast} tone="mint" /> : null}
        </Surface>
      </div>
    </div>
  );
}
