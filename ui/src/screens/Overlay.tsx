import type { StyleDesc } from "@gpuix/react";
import { BrandMark, Button, StatusDot } from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

/**
 * The overlay is the surface people read during a call: caption size and the
 * two-tier choice come from Settings, the content is centred while sparse, and
 * every control carries a visible word.
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
  const latest = state.segments[state.segments.length - 1];

  const shell: StyleDesc = {
    height: "100%",
    padding: 12,
    display: "flex",
    flexDirection: "column",
    gap: 8,
    backgroundColor: "#0D0D12F2",
    borderWidth: 1,
    borderColor: "#FFFFFF24",
    borderRadius: 12,
  };

  return (
    <div style={{ height: "100%", padding: 8 }}>
      <div style={shell} role="group" aria-label={t(locale, "Caption overlay")}>
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
          <BrandMark size={20} />
          <text style={textStyle(type.meta, colors.text, 600)}>{language.shortName}</text>
          <StatusDot
            tone={state.phase === "paused" ? "amber" : "mint"}
            label={state.phase === "paused" ? t(locale, "Paused") : t(locale, "Listening")}
          />
          {state.reversionActive ? (
            <text style={textStyle(type.meta, colors.faint)}>EN</text>
          ) : null}
          <div style={{ flexGrow: 1 }} />
          <Button
            label={t(locale, "Move window")}
            kind="ghost"
            onMouseDown={onDrag}
            style={{ height: 28, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Settings")}
            kind="ghost"
            onClick={onSettings}
            style={{ height: 28, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Expand")}
            kind="ghost"
            onClick={onExpand}
            style={{ height: 28, paddingLeft: 8, paddingRight: 8 }}
          />
          <Button
            label={t(locale, "Console")}
            kind="ghost"
            onClick={onConsole}
            style={{ height: 28, paddingLeft: 8, paddingRight: 8 }}
          />
        </div>

        <div
          style={{
            flexGrow: 1,
            minHeight: 0,
            display: "flex",
            flexDirection: "column",
            gap: 5,
            justifyContent: "center",
          }}
        >
          {state.partial ? (
            <div role="group" aria-label={t(locale, "Listening — keep speaking")}>
              <text style={textStyle(type.body, colors.muted, 500)}>{state.partial.text}</text>
            </div>
          ) : null}
          {latest ? (
            <div
              role="button"
              tabIndex={0}
              aria-label={latest.text}
              onClick={() => onSimplify(latest.id)}
              onKeyDown={(event) => {
                if (event.key === "enter" || event.key === "space" || event.key === " ") {
                  event.preventDefault();
                  onSimplify(latest.id);
                }
              }}
              style={{ display: "flex", flexDirection: "column", gap: 4, cursor: "pointer" }}
            >
              {showTier1 ? (
                <text style={textStyle(Math.max(14, Math.round(captionSize * 0.72)), colors.muted)}>
                  {latest.text}
                </text>
              ) : null}
              <text
                style={{
                  ...textStyle(captionSize, colors.text, 600),
                  lineHeight: Math.round(captionSize * 1.3),
                }}
              >
                {latest.simplified ?? t(locale, "Tap a sentence to simplify it")}
              </text>
            </div>
          ) : (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 4,
                flexGrow: 1,
                justifyContent: "center",
              }}
            >
              <text style={{ ...textStyle(captionSize, colors.text, 600), lineHeight: 24 }}>
                {t(locale, "Listening — keep speaking")}
              </text>
              <text style={textStyle(type.meta, colors.faint)}>
                {t(locale, "Start speaking, or use the demo conversation.")}
              </text>
            </div>
          )}
        </div>

        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
          <text style={textStyle(type.meta, colors.faint)}>
            {latest?.simplified ? t(locale, "Simplified caption") : t(locale, "Verbatim")}
          </text>
          <div style={{ flexGrow: 1 }} />
          <Button
            label={state.phase === "paused" ? t(locale, "Resume") : t(locale, "Pause")}
            kind="ghost"
            onClick={state.phase === "paused" ? onResume : onPause}
          />
          <text style={textStyle(type.meta, colors.faint)}>{t(locale, "click to simplify")}</text>
        </div>
      </div>
    </div>
  );
}
