import { useEffect, useState } from "react";
import {
  BrandMark,
  Button,
  CaptionRow,
  StatusDot,
  Surface,
  Tabs,
  Toast,
} from "../aurora/components";
import { colors, focusRing, textStyle, type } from "../aurora/theme";
import {
  captionKeyAction,
  isReadLatestShortcut,
  latestCaptionIndex,
  READ_LATEST_SHORTCUT,
  windowKeys,
} from "../captionKeys";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import type { SessionView } from "../store/sessionStore";

type WindowMode = "console" | "overlay" | "expanded";

/**
 * Running face, Mac cadence: headline, one line that teaches speak -> pause ->
 * click, the transcript as the only content, and the session cluster at the
 * bottom. No per-row badges or timestamps; the simplified line is the caption.
 */
export function LiveScreen({
  state,
  locale,
  captionSize,
  showTier1,
  windowMode = "console",
  onSimplify,
  onStop,
  onPause,
  onResume,
  onSettings,
  onWindowMode,
  onTypedSimplify,
  compact = false,
  simplifyingId = null,
}: {
  state: SessionView;
  locale: string;
  captionSize: number;
  showTier1: boolean;
  windowMode?: WindowMode;
  onSimplify: (id: string) => void;
  onStop: () => void;
  onPause: () => void;
  onResume: () => void;
  onSettings: () => void;
  onWindowMode: (mode: WindowMode) => void;
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
  const needsTypedInput = language.asrStatus !== "ok";
  const hearing = Boolean(state.partial?.text);

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

  const cadence =
    state.phase === "paused"
      ? t(locale, "Paused — click a sentence to simplify it")
      : simplifyingId
        ? t(locale, "Simplifying…")
        : hearing
          ? t(locale, "Listening — keep speaking")
          : t(locale, "Waiting for speech");

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 12,
        padding: compact ? 14 : 20,
        height: "100%",
      }}
    >
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
        <BrandMark size={compact ? 24 : 28} />
        <text style={{ ...textStyle(type.meta, colors.faint, 700) }}>
          {t(locale, "LIVE LINGUIST · ON-DEVICE")}
        </text>
        <div style={{ flexGrow: 1 }} />
        {!compact ? (
          <>
            <Button
              label={`${t(locale, "Read latest")} (${READ_LATEST_SHORTCUT})`}
              kind="ghost"
              onClick={readLatest}
            />
            <Button label={t(locale, "Settings")} kind="secondary" onClick={onSettings} />
          </>
        ) : null}
      </div>

      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
        <text style={{ ...textStyle(compact ? 24 : 32, colors.text, 600), lineHeight: 38 }}>
          {state.phase === "paused" ? t(locale, "Paused") : t(locale, "Captioning")}
        </text>
        <div style={{ flexGrow: 1 }} />
        {!compact ? (
          <Tabs
            tabs={[
              { id: "console", label: t(locale, "Console window") },
              { id: "overlay", label: t(locale, "Caption overlay") },
              { id: "expanded", label: t(locale, "Expanded transcript") },
            ]}
            value={windowMode}
            onChange={(id) => onWindowMode(id as WindowMode)}
          />
        ) : null}
      </div>

      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 7 }}>
        <StatusDot
          tone={state.phase === "paused" ? "amber" : hearing ? "mint" : "neutral"}
          label={cadence}
        />
        {state.reversionActive ? <StatusDot tone="amber" label="EN" /> : null}
        {state.overrunCount > 0 ? (
          <text style={textStyle(type.meta, colors.rose)}>
            {t(locale, "overrun %@", state.overrunCount)}
          </text>
        ) : null}
      </div>

      <Surface
        style={{
          padding: compact ? 8 : 12,
          flexGrow: 1,
          display: "flex",
          flexDirection: "column",
          gap: 8,
          minHeight: 0,
        }}
      >
        {state.segments.length === 0 ? (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 8,
              padding: compact ? 8 : 20,
            }}
          >
            <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 20 }}>
              {t(locale, "Start speaking — your sentences appear here as a transcript.")}
            </text>
            <text style={{ ...textStyle(type.meta, colors.faint), lineHeight: 18 }}>
              {t(locale, "Pause any time and click a sentence to see it in simpler words.")}
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
                targetLanguage={state.language}
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
              borderRadius: 8,
              backgroundColor: "#FFFFFF06",
              display: "flex",
              flexDirection: "row",
              gap: 8,
              alignItems: "center",
            }}
          >
            <div style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: colors.mint }} />
            <text style={{ ...textStyle(compact ? type.meta : 15, colors.muted), flexGrow: 1 }}>
              {state.partial.text}
            </text>
          </div>
        ) : null}
      </Surface>

      {!compact && needsTypedInput ? (
        <Surface style={{ padding: 12, display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
            <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
              {t(locale, "TEXT TO SIMPLIFY")}
            </text>
            <StatusDot tone="amber" label={statusLabel(language.asrStatus)} />
          </div>
          <div style={{ display: "flex", flexDirection: "row", gap: 8, alignItems: "flex-start" }}>
            <textarea
              value={typed}
              placeholder={t(locale, "Paste or type text to simplify it…")}
              onChange={(event) => setTyped(event.value ?? "")}
              minRows={2}
              maxRows={4}
              style={{
                flexGrow: 1,
                minWidth: 0,
                fontSize: type.label,
                lineHeight: 19,
                color: colors.text,
                backgroundColor: "#00000033",
                borderWidth: 1,
                borderColor: colors.border,
                borderRadius: 8,
                padding: 8,
                focusVisible: focusRing,
              }}
            />
            <Button
              label={t(locale, "Simplify")}
              kind="secondary"
              onClick={() => void submitTyped()}
            />
          </div>
          {typedResult ? (
            <text style={{ ...textStyle(type.body, colors.text, 600), lineHeight: 20 }}>
              {typedResult}
            </text>
          ) : null}
        </Surface>
      ) : null}

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 8,
        }}
      >
        <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
          {state.phase === "paused" ? (
            <Button label={t(locale, "Resume captions")} onClick={onResume} />
          ) : (
            <Button label={t(locale, "Pause captions")} kind="secondary" onClick={onPause} />
          )}
          <Button label={t(locale, "Stop")} kind="danger" onClick={onStop} />
        </div>
        <text style={textStyle(type.meta, colors.faint)}>
          {t(locale, "Offline — audio never leaves this device")}
        </text>
      </div>
      {toast ? <Toast message={toast} tone="info" /> : null}
    </div>
  );
}
