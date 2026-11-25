import type { StyleDesc } from "@gpuix/react";
import { Badge, BrandMark, Button, StatusDot } from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

/**
 * Mac caption panel: the live partial above the last finalized lines, in
 * black glass, sized from the caption-size setting. A single quiet control row
 * exists only because the PC overlay is the app's only window and must offer
 * a way back.
 */
export function OverlayScreen({
  state,
  locale,
  captionSize,
  showTier1,
  onSimplify,
  onPause,
  onResume,
  onSettings,
  onDrag,
  onExpand,
  onConsole,
}: {
  state: SessionView;
  locale: string;
  captionSize: number;
  showTier1: boolean;
  onSimplify: (id: string) => void;
  onPause: () => void;
  onResume: () => void;
  onSettings: () => void;
  onDrag: () => void;
  onExpand: () => void;
  onConsole: () => void;
}) {
  const language = languageByCode(state.language);
  const lines = state.segments.slice(-2);
  const latest = state.segments[state.segments.length - 1];
  const partialSize = Math.max(13, Math.round(captionSize * 0.55));

  const shell: StyleDesc = {
    height: "100%",
    padding: 16,
    display: "flex",
    flexDirection: "column",
    gap: 8,
    backgroundColor: "#0A0A0ACC",
    borderWidth: 1,
    borderColor: state.reversionActive ? "#F5C06A99" : "#FFFFFF24",
    borderRadius: 16,
  };

  return (
    <div style={{ height: "100%", padding: 8 }}>
      <div style={shell} role="group" aria-label={t(locale, "Caption overlay")}>
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
          <BrandMark size={20} />
          <text style={textStyle(type.meta, colors.muted, 600)}>{language.shortName}</text>
          <StatusDot
            tone={state.phase === "paused" ? "amber" : "mint"}
            label={state.phase === "paused" ? t(locale, "Paused") : t(locale, "Listening")}
          />
          {state.reversionActive ? <Badge label="EN" tone="amber" /> : null}
          <div style={{ flexGrow: 1 }} />
          <Button
            label={state.phase === "paused" ? t(locale, "Resume") : t(locale, "Pause")}
            kind="ghost"
            onClick={state.phase === "paused" ? onResume : onPause}
            style={{ height: 26, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Move window")}
            kind="ghost"
            onMouseDown={onDrag}
            style={{ height: 26, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Settings")}
            kind="ghost"
            onClick={onSettings}
            style={{ height: 26, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Expand")}
            kind="ghost"
            onClick={onExpand}
            style={{ height: 26, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Console window")}
            kind="ghost"
            onClick={onConsole}
            style={{ height: 26, paddingLeft: 8, paddingRight: 8 }}
          />
        </div>

        <div
          role="button"
          tabIndex={0}
          aria-label={latest ? latest.text : t(locale, "Listening — keep speaking")}
          onClick={() => latest && onSimplify(latest.id)}
          onKeyDown={(event) => {
            if (!latest) return;
            if (event.key === "enter" || event.key === "space" || event.key === " ") {
              event.preventDefault();
              onSimplify(latest.id);
            }
          }}
          style={{
            flexGrow: 1,
            minHeight: 0,
            display: "flex",
            flexDirection: "column",
            gap: 6,
            justifyContent: "center",
            cursor: latest ? "pointer" : "default",
          }}
        >
          {showTier1 && state.partial?.text ? (
            <text
              style={{
                ...textStyle(partialSize, colors.muted),
                lineHeight: Math.round(partialSize * 1.35),
                whiteSpace: "normal",
                lineClamp: 2,
              }}
            >
              {state.partial.text}
            </text>
          ) : null}

          {lines.length === 0 ? (
            <text style={{ ...textStyle(captionSize, colors.text, 600), lineHeight: 28 }}>
              {t(locale, "Listening — keep speaking")}
            </text>
          ) : (
            lines.map((line) => (
              <text
                key={line.id}
                style={{
                  ...textStyle(
                    line.simplified ? captionSize : Math.round(captionSize * 0.8),
                    colors.text,
                    600,
                  ),
                  lineHeight: Math.round(
                    (line.simplified ? captionSize : captionSize * 0.8) * 1.32,
                  ),
                  lineClamp: 2,
                }}
              >
                {line.simplified ?? line.text}
              </text>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
