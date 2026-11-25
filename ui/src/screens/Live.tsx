import { useEffect, useState } from "react";
import {
  Badge,
  Button,
  CaptionRow,
  Divider,
  SectionHeader,
  Surface,
  Tabs,
  Toast,
} from "../aurora/components";
import { colors, focusRing, gradient, textStyle } from "../aurora/theme";
import {
  captionKeyAction,
  isReadLatestShortcut,
  latestCaptionIndex,
  READ_LATEST_SHORTCUT,
  windowKeys,
} from "../captionKeys";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView, SimplificationMode } from "../store/sessionStore";

export function LiveScreen({
  state,
  locale,
  captionSize,
  showTier1,
  onSimplify,
  onStop,
  onPause,
  onResume,
  onSettings,
  onMode,
  onWindowMode,
  onTypedSimplify,
  compact = false,
  simplifyingId = null,
}: {
  state: SessionView;
  locale: string;
  captionSize: number;
  showTier1: boolean;
  onSimplify: (id: string) => void;
  onStop: () => void;
  onPause: () => void;
  onResume: () => void;
  onSettings: () => void;
  onMode: (mode: SimplificationMode) => void;
  onWindowMode: (mode: "console" | "overlay" | "expanded") => void;
  onTypedSimplify: (
    text: string,
  ) => Promise<{ text: string | null; status: string; flags: string[] }>;
  compact?: boolean;
  simplifyingId?: string | null;
}) {
  const language = languageByCode(state.language);
  const [typed, setTyped] = useState("");
  const [typedResult, setTypedResult] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const latest =
    [...state.segments].reverse().find((segment) => segment.simplified) ??
    state.segments[state.segments.length - 1];
  const simplifiedCount = state.segments.filter((segment) => segment.simplified).length;

  // Keyboard focus: GPUIX has no imperative focus(), so the focused row is
  // remounted with autoFocus (key carries a nonce) when focus should move.
  const [focus, setFocus] = useState<{ index: number; nonce: number }>({ index: -1, nonce: 0 });
  const moveFocus = (index: number) => setFocus((current) => ({ index, nonce: current.nonce + 1 }));

  const onCaptionKey = (index: number, key: string): boolean => {
    const result = captionKeyAction(index, state.segments.length, key);
    if (result.action === "move") {
      if (result.index !== index) moveFocus(result.index);
      return true;
    }
    if (result.action === "simplify") {
      const segment = state.segments[result.index];
      if (segment) onSimplify(segment.id);
      return true;
    }
    return false;
  };

  // Read latest: show it and move keyboard focus to that caption row so the
  // screen reader reads the row's label (GPUIX has no live-region announce).
  const readLatest = () => {
    if (!latest) {
      setToast(t(locale, "Waiting for speech"));
      return;
    }
    setToast(`${latest.simplified ?? latest.text}`);
    moveFocus(latestCaptionIndex(state.segments));
  };

  useEffect(() =>
    windowKeys.subscribe((key, modifiers) => {
      if (!isReadLatestShortcut(key, modifiers)) return false;
      readLatest();
      return true;
    }),
  );

  const submitTyped = async () => {
    if (!typed.trim()) return;
    const result = await onTypedSimplify(typed);
    setTypedResult(result.text ?? t(locale, "Shown without simplification"));
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: compact ? "column" : "row",
        gap: compact ? 10 : 20,
        padding: compact ? 14 : 24,
        height: "100%",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 14, flexGrow: 1, minWidth: 0 }}>
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: 10,
              background: gradient("#8B7CFF", "#2ED3B7", 135),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <text style={textStyle(12, "#0B0B12", 800)}>LL</text>
          </div>
          <div
            style={{ display: "flex", flexDirection: "column", gap: 1, flexGrow: 1, minWidth: 0 }}
          >
            <text style={textStyle(compact ? 14 : 18, colors.text, 700)}>
              {t(locale, "CAPTIONS")}
            </text>
            <text style={textStyle(10, colors.faint)}>
              {language.name} ·{" "}
              {state.mode === "on_demand" ? t(locale, "On-demand") : t(locale, "Live")}
            </text>
          </div>
          <Badge
            label={state.phase === "paused" ? t(locale, "Paused") : t(locale, "Listening")}
            tone={state.phase === "paused" ? "amber" : "mint"}
          />
          {state.reversionActive ? (
            <Badge label={t(locale, "English reversion")} tone="info" />
          ) : null}
          {state.overrunCount > 0 ? (
            <Badge label={`overrun ${state.overrunCount}`} tone="rose" />
          ) : null}
          {!compact ? (
            <div style={{ display: "flex", flexDirection: "row", gap: 6 }}>
              <Button
                label={`${t(locale, "Read latest")} (${READ_LATEST_SHORTCUT})`}
                kind="ghost"
                onClick={readLatest}
              />
              <Button label={t(locale, "Settings")} kind="secondary" onClick={onSettings} />
            </div>
          ) : null}
        </div>

        {!compact ? (
          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
            <Tabs
              tabs={[
                { id: "on_demand", label: t(locale, "On-demand") },
                { id: "live", label: t(locale, "Live") },
              ]}
              value={state.mode}
              onChange={(id) => onMode(id as SimplificationMode)}
            />
            <div style={{ flexGrow: 1 }} />
            <Button
              label={t(locale, "Console window")}
              kind="ghost"
              onClick={() => onWindowMode("console")}
            />
            <Button
              label={t(locale, "Caption overlay")}
              kind="ghost"
              onClick={() => onWindowMode("overlay")}
            />
            <Button
              label={t(locale, "Expanded transcript")}
              kind="ghost"
              onClick={() => onWindowMode("expanded")}
            />
          </div>
        ) : null}

        <Surface
          style={{
            padding: compact ? 10 : 14,
            flexGrow: 1,
            display: "flex",
            flexDirection: "column",
            gap: 10,
            minHeight: 0,
            backgroundColor: compact ? "#0F0F18F2" : "#0F0F18AA",
          }}
        >
          {state.segments.length === 0 ? (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 8,
                padding: compact ? 8 : 24,
                flexGrow: 1,
                justifyContent: "center",
                alignItems: "center",
              }}
            >
              <text style={textStyle(compact ? 16 : 22, colors.text, 700)}>
                {t(locale, "Start speaking — your sentences appear here as a transcript.")}
              </text>
              <text style={textStyle(12, colors.muted)}>
                {state.mode === "on_demand"
                  ? t(locale, "Pause any time and click a sentence to see it in simpler words.")
                  : t(locale, "Simplified captions always stay visible — they are the point.")}
              </text>
            </div>
          ) : (
            <div
              style={{
                flexGrow: 1,
                minHeight: 0,
                overflowY: "scroll",
                overflowX: "hidden",
                scrollbarColor: "#FFFFFF22",
                scrollbarWidth: 6,
                display: "flex",
                flexDirection: "column",
                gap: compact ? 6 : 10,
                paddingRight: 4,
              }}
            >
              {state.segments.map((segment, index) => (
                <CaptionRow
                  key={index === focus.index ? `${segment.id}-f${focus.nonce}` : segment.id}
                  segment={segment}
                  onSimplify={onSimplify}
                  onKey={(key) => onCaptionKey(index, key)}
                  autoFocus={index === focus.index}
                  simplifying={segment.id === simplifyingId}
                  locale={locale}
                  compact={compact}
                  captionSize={captionSize}
                  showTier1={showTier1}
                />
              ))}
            </div>
          )}
          {state.partial ? (
            <div
              role="group"
              aria-label={t(locale, "Listening — keep speaking")}
              style={{
                paddingTop: 8,
                paddingBottom: 8,
                paddingLeft: 10,
                paddingRight: 10,
                borderRadius: 10,
                backgroundColor: "#FFFFFF06",
                borderWidth: 1,
                borderColor: colors.border,
                display: "flex",
                flexDirection: "row",
                gap: 8,
                alignItems: "center",
              }}
            >
              <div style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: colors.mint }} />
              <text style={{ ...textStyle(compact ? 12 : 14, colors.muted), flexGrow: 1 }}>
                {state.partial.text}
              </text>
              <text style={textStyle(10, colors.faint)}>
                {t(locale, "Listening — keep speaking")}
              </text>
            </div>
          ) : null}
        </Surface>

        {!compact ? (
          <div style={{ display: "flex", flexDirection: "row", gap: 10, alignItems: "center" }}>
            {state.phase === "paused" ? (
              <Button label={t(locale, "Resume captions")} onClick={onResume} />
            ) : (
              <Button label={t(locale, "Pause captions")} kind="secondary" onClick={onPause} />
            )}
            <Button label={t(locale, "Stop")} kind="danger" onClick={onStop} />
            <div style={{ flexGrow: 1 }} />
            <Badge
              label={`${simplifiedCount}/${state.segments.length} simplified`}
              tone="neutral"
            />
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "row", gap: 8, alignItems: "center" }}>
            <Button
              label={state.phase === "paused" ? t(locale, "Resume") : t(locale, "Pause")}
              kind="secondary"
              onClick={state.phase === "paused" ? onResume : onPause}
            />
            <Button
              label={t(locale, "Expand")}
              kind="ghost"
              onClick={() => onWindowMode("expanded")}
            />
            <div style={{ flexGrow: 1 }} />
            <text style={textStyle(10, colors.faint)}>
              {simplifiedCount}/{state.segments.length}
            </text>
          </div>
        )}
        {toast ? <Toast message={toast} tone="info" /> : null}
      </div>

      {!compact ? (
        <div
          style={{ width: 290, flexShrink: 0, display: "flex", flexDirection: "column", gap: 14 }}
        >
          <Surface style={{ padding: 16, display: "flex", flexDirection: "column", gap: 12 }}>
            <SectionHeader eyebrow={t(locale, "SESSION")} title={t(locale, "Live transcript")} />
            <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
              <Stat label={t(locale, "SENTENCES")} value={String(state.segments.length)} />
              <Stat label={t(locale, "SIMPLIFIED")} value={String(simplifiedCount)} />
            </div>
            <Divider />
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
              <text style={textStyle(11, colors.muted)}>{t(locale, "ASR status")}</text>
              <div style={{ flexGrow: 1 }} />
              <Badge
                label={statusLabel(language.asrStatus)}
                tone={language.asrStatus === "ok" ? "mint" : "amber"}
              />
            </div>
            {language.asrStatus === "ok" ? null : (
              <text style={textStyle(11, colors.faint)}>
                {t(
                  locale,
                  "Live listening is not available for this language yet. Paste or type text to simplify it.",
                )}
              </text>
            )}
          </Surface>

          <Surface style={{ padding: 16, display: "flex", flexDirection: "column", gap: 10 }}>
            <text style={{ ...textStyle(10, colors.faint, 700) }}>
              {t(locale, "SIMPLIFY-ONLY INPUT")}
            </text>
            <textarea
              value={typed}
              placeholder={t(locale, "Paste or type text to simplify it…")}
              onChange={(event) => setTyped(event.value ?? "")}
              minRows={3}
              maxRows={6}
              style={{
                fontSize: 12,
                lineHeight: 18,
                color: colors.text,
                backgroundColor: "#00000033",
                borderWidth: 1,
                borderColor: colors.border,
                borderRadius: 10,
                padding: 8,
                focusVisible: focusRing,
              }}
            />
            <Button
              label={t(locale, "Simplify")}
              kind="secondary"
              onClick={() => void submitTyped()}
            />
            {typedResult ? <text style={textStyle(13, "#E4DFFF", 600)}>{typedResult}</text> : null}
          </Surface>

          <Surface style={{ padding: 16, display: "flex", flexDirection: "column", gap: 8 }}>
            <text style={{ ...textStyle(10, colors.faint, 700) }}>{t(locale, "PRIVACY")}</text>
            <text style={textStyle(11, colors.muted)}>
              {t(locale, "Offline — audio never leaves this device")}
            </text>
            <text style={textStyle(11, colors.faint)}>
              {t(locale, "Transcripts are kept in memory only. Export now to keep one.")}
            </text>
          </Surface>
        </div>
      ) : null}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        flexGrow: 1,
        minWidth: 0,
        padding: 8,
        borderRadius: 8,
        backgroundColor: "#00000033",
        display: "flex",
        flexDirection: "column",
        gap: 2,
      }}
    >
      <text style={{ ...textStyle(9, colors.faint, 700) }}>{label}</text>
      <text style={textStyle(18, colors.text, 700)}>{value}</text>
    </div>
  );
}
