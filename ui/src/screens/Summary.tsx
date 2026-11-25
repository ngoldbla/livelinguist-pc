import { useState } from "react";
import {
  Button,
  CaptionRow,
  Divider,
  PageHeader,
  StatusDot,
  SummaryTile,
  Surface,
  Toast,
} from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
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
  const inEnglish = state.segments.filter((segment) => segment.detectedLanguage === "en").length;

  const exportAs = async (format: "markdown" | "plain" | "csv") => {
    try {
      await onExport(format);
      setToast(`${t(locale, "Export transcript")} · ${format}`);
    } catch {
      setToast("Export failed. Check the destination path.");
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, padding: 24, height: "100%" }}>
      <PageHeader
        backLabel={t(locale, "Back")}
        title={t(locale, "Session summary")}
        subtitle={new Date().toLocaleDateString(locale)}
        onBack={onBack}
        trailing={
          <StatusDot
            tone={state.phase === "summary" ? "mint" : "amber"}
            label={state.phase === "summary" ? t(locale, "Done") : t(locale, "Listening")}
          />
        }
      />

      <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
        <SummaryTile
          label={t(locale, "DURATION")}
          value={`${minutes}:${String(seconds).padStart(2, "0")}`}
        />
        <SummaryTile
          label={t(locale, "SENTENCES")}
          value={String(state.segments.length)}
          hint={`${simplified} ${t(locale, "SIMPLIFIED").toLowerCase()}`}
        />
        <SummaryTile
          label={t(locale, "IN TARGET LANGUAGE")}
          value={targetPercent == null ? "—" : `${targetPercent}%`}
          hint={state.language}
          tone={colors.mint}
        />
        <SummaryTile
          label={t(locale, "ENGLISH REVERSIONS")}
          value={String(inEnglish)}
          hint={t(locale, "sentences in English")}
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
          <div style={{ display: "flex", flexDirection: "row", alignItems: "baseline", gap: 10 }}>
            <text style={{ ...textStyle(type.title, colors.text, 600), flexGrow: 1 }}>
              {t(locale, "Transcript")}
            </text>
            <text style={textStyle(type.meta, colors.faint)}>
              {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
            </text>
          </div>
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
              <text style={textStyle(type.body, colors.muted)}>
                {t(locale, "No session data.")}
              </text>
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
            alignSelf: "flex-start",
            padding: 18,
            display: "flex",
            flexDirection: "column",
            gap: 10,
          }}
        >
          <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
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
          <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
            {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
          </text>
          <Divider />
          <Button label={t(locale, "Start another session")} onClick={onConsole} />
          {toast ? <Toast message={toast} tone="mint" /> : null}
        </Surface>
      </div>
    </div>
  );
}
