import type { StyleDesc } from "@gpuix/react";
import { Badge, Button } from "../aurora/components";
import { colors, gradient, textStyle } from "../aurora/theme";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

export function OverlayScreen({
  state,
  locale,
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
  const simplified = [...state.segments].reverse().find((segment) => segment.simplified);

  const shell: StyleDesc = {
    height: "100%",
    padding: 14,
    display: "flex",
    flexDirection: "column",
    gap: 8,
    background: gradient("#0B0B12F2", "#15102AF2", 120),
    borderWidth: 1,
    borderColor: "#FFFFFF22",
    borderRadius: 16,
    boxShadow: { offsetX: 0, offsetY: 12, blurRadius: 40, spreadRadius: 0, color: "#00000088" },
  };

  return (
    <div style={{ height: "100%", padding: 10 }}>
      <div style={shell}>
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
          <div
            style={{
              width: 22,
              height: 22,
              borderRadius: 7,
              background: gradient("#8B7CFF", "#2ED3B7", 135),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <text style={textStyle(8, "#0B0B12", 800)}>LL</text>
          </div>
          <text style={textStyle(11, colors.text, 700)}>{language.shortName}</text>
          <Badge
            label={state.phase === "paused" ? t(locale, "Paused") : t(locale, "Listening")}
            tone={state.phase === "paused" ? "amber" : "mint"}
          />
          {state.reversionActive ? <Badge label="EN" tone="info" /> : null}
          <div style={{ flexGrow: 1 }} />
          <text style={textStyle(9, colors.faint)}>{statusLabel(language.asrStatus)}</text>
          <OverlayIcon
            glyph="⋮⋮"
            label="Move window"
            onClick={() => {}}
            onMouseDown={onDrag}
            focusable={false}
          />
          <OverlayIcon glyph="⟳" label={t(locale, "Settings")} onClick={onSettings} />
          <OverlayIcon glyph="⤢" label={t(locale, "Expanded transcript")} onClick={onExpand} />
          <OverlayIcon glyph="▣" label={t(locale, "Console window")} onClick={onConsole} />
        </div>

        <div
          style={{ flexGrow: 1, minHeight: 0, display: "flex", flexDirection: "column", gap: 5 }}
        >
          {latest ? (
            <>
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
                style={{ display: "flex", flexDirection: "column", gap: 3, cursor: "pointer" }}
              >
                <text style={textStyle(11, colors.muted)}>{latest.text}</text>
                <text style={textStyle(17, colors.text, 600)}>
                  {latest.simplified ?? t(locale, "Tap a sentence to simplify it")}
                </text>
              </div>
              {latest.flags.length > 0 ? (
                <div style={{ display: "flex", flexDirection: "row", gap: 5 }}>
                  {latest.flags.slice(0, 2).map((flag) => (
                    <Badge key={flag} label={flag} tone="amber" />
                  ))}
                </div>
              ) : null}
            </>
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
              <text style={textStyle(17, colors.text, 600)}>
                {t(locale, "Listening — keep speaking")}
              </text>
              <text style={textStyle(11, colors.faint)}>
                {t(locale, "Start speaking, or use the demo conversation.")}
              </text>
            </div>
          )}
        </div>

        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
          <text style={textStyle(9, colors.faint)}>
            {simplified ? t(locale, "Simplified caption") : t(locale, "Verbatim")}
          </text>
          <div style={{ flexGrow: 1 }} />
          <Button
            label={state.phase === "paused" ? t(locale, "Resume") : t(locale, "Pause")}
            kind="ghost"
            onClick={state.phase === "paused" ? onResume : onPause}
          />
          <text style={textStyle(9, colors.faint)}>{t(locale, "click to simplify")}</text>
        </div>
      </div>
    </div>
  );
}

function OverlayIcon({
  glyph,
  label,
  onClick,
  onMouseDown,
  focusable = true,
}: {
  glyph: string;
  label: string;
  onClick: () => void;
  onMouseDown?: () => void;
  focusable?: boolean;
}) {
  return (
    <div
      role={focusable ? "button" : undefined}
      tabIndex={focusable ? 0 : undefined}
      aria-label={focusable ? label : undefined}
      aria-hidden={focusable ? undefined : true}
      onClick={onClick}
      onMouseDown={onMouseDown}
      onKeyDown={
        focusable
          ? (event) => {
              if (event.key === "enter" || event.key === "space" || event.key === " ") {
                event.preventDefault();
                onClick();
              }
            }
          : undefined
      }
      style={{
        width: 22,
        height: 22,
        borderRadius: 7,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "#FFFFFF0A",
        cursor: "pointer",
        hover: { backgroundColor: "#FFFFFF16" },
      }}
    >
      <text style={textStyle(10, colors.muted, 700)}>{glyph}</text>
    </div>
  );
}
