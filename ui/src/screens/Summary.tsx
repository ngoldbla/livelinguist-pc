import { useState } from "react";
import { BrandMark, Button, Divider, SummaryTile, Surface, Toast } from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

/**
 * Mac summary sheet: brand, title, stat tiles, the memory-only note, exports
 * and Done. No transcript list; the live screen was the transcript and export
 * is how a session is kept.
 */
export function SummaryScreen({
  state,
  locale,
  onExport,
  onDone,
}: {
  state: SessionView;
  locale: string;
  onExport: (format: "markdown" | "plain" | "csv") => Promise<void>;
  onDone: () => void;
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
    <div style={{ height: "100%", display: "flex", justifyContent: "center", overflow: "scroll" }}>
      <div
        style={{
          width: "100%",
          maxWidth: 560,
          display: "flex",
          flexDirection: "column",
          gap: 18,
          padding: 28,
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 10,
            paddingTop: 8,
          }}
        >
          <BrandMark size={44} />
          <text
            style={{
              ...textStyle(type.page, colors.text, 700),
              textAlign: "center",
            }}
          >
            {t(locale, "Session summary")}
          </text>
        </div>

        <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
          <SummaryTile
            label={t(locale, "DURATION")}
            value={t(locale, "%lld min %lld s", minutes, seconds)}
          />
          <SummaryTile label={t(locale, "LANGUAGE")} value={languageByCode(state.language).name} />
        </div>
        <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
          <SummaryTile label={t(locale, "SENTENCES")} value={String(state.segments.length)} />
          <SummaryTile
            label={t(locale, "IN TARGET LANGUAGE")}
            value={targetPercent == null ? "—" : `${targetPercent}%`}
            tone={colors.mint}
          />
        </div>
        {inEnglish > 0 ? (
          <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
            <SummaryTile
              label={t(locale, "ENGLISH REVERSIONS")}
              value={String(inEnglish)}
              tone={colors.amber}
            />
            <div style={{ flexGrow: 1 }} />
          </div>
        ) : null}

        <Surface style={{ padding: 14 }}>
          <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
            {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
          </text>
        </Surface>

        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
            <Button
              label={t(locale, "Markdown")}
              icon="download"
              kind="secondary"
              style={{ flexGrow: 1 }}
              onClick={() => void exportAs("markdown")}
            />
            <Button
              label={t(locale, "Plain text (.txt)")}
              kind="secondary"
              style={{ flexGrow: 1 }}
              onClick={() => void exportAs("plain")}
            />
            <Button
              label={t(locale, "Diagnostics CSV")}
              kind="secondary"
              style={{ flexGrow: 1 }}
              onClick={() => void exportAs("csv")}
            />
          </div>
          <Divider />
          <Button label={t(locale, "Done")} icon="check" onClick={onDone} style={{ height: 44 }} />
        </div>
        {toast ? <Toast message={toast} tone="mint" /> : null}
      </div>
    </div>
  );
}
