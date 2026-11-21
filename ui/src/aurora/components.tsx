import type { StyleDesc } from "@gpuix/react";
import type { ReactNode } from "react";
import type { Segment } from "../store/sessionStore";
import { card, colors, focusRing, glass, gradient, textStyle } from "./theme";

export function AuroraField({ active = false }: { active?: boolean }) {
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        overflow: "hidden",
        background: gradient("#08080F", "#17102B", 145),
      }}
    >
      <div
        style={{
          position: "absolute",
          top: -260,
          left: -180,
          width: 760,
          height: 760,
          borderRadius: 380,
          background: gradient(active ? "#5B4BDB88" : "#3D347A55", "#00000000", 90),
        }}
      />
      <div
        style={{
          position: "absolute",
          bottom: -320,
          right: -160,
          width: 860,
          height: 860,
          borderRadius: 430,
          background: gradient(active ? "#1CC9B866" : "#1B7D7455", "#00000000", 300),
        }}
      />
      <div
        style={{
          position: "absolute",
          top: 280,
          left: 440,
          width: 500,
          height: 500,
          borderRadius: 250,
          background: gradient(active ? "#FFB86B22" : "#8B7CFF1A", "#00000000", 30),
        }}
      />
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          height: 1,
          backgroundColor: "#FFFFFF18",
        }}
      />
    </div>
  );
}

export function Surface({
  children,
  style,
  padding = 20,
}: {
  children: ReactNode;
  style?: StyleDesc;
  padding?: number;
}) {
  return <div style={{ ...glass, padding, ...style }}>{children}</div>;
}

export const GlassCard = Surface;

export function SectionHeader({
  eyebrow,
  title,
  subtitle,
  trailing,
}: {
  eyebrow?: string;
  title: string;
  subtitle?: string;
  trailing?: ReactNode;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "flex-end", gap: 16 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 6, flexGrow: 1, minWidth: 0 }}>
        {eyebrow ? (
          <text style={{ ...textStyle(11, colors.accent, 700) }}>{eyebrow.toUpperCase()}</text>
        ) : null}
        <text style={{ ...textStyle(26, colors.text, 700), lineHeight: 32 }}>{title}</text>
        {subtitle ? <text style={textStyle(13, colors.muted)}>{subtitle}</text> : null}
      </div>
      {trailing}
    </div>
  );
}

type Tone = "neutral" | "accent" | "mint" | "amber" | "rose" | "info";

const toneStyles: Record<Tone, { bg: string; fg: string; border: string }> = {
  neutral: { bg: "#FFFFFF0D", fg: colors.muted, border: colors.border },
  accent: { bg: colors.accentSoft, fg: "#C9C1FF", border: "#8B7CFF44" },
  mint: { bg: colors.mintSoft, fg: "#A7F4E7", border: "#59E1C844" },
  amber: { bg: colors.amberSoft, fg: colors.amber, border: "#FFC46B44" },
  rose: { bg: colors.roseSoft, fg: "#FFB5C1", border: "#FF7A9044" },
  info: { bg: "#4EA8FF1F", fg: "#9BCDFF", border: "#4EA8FF44" },
};

export function Badge({
  label,
  tone = "neutral",
  testId,
}: {
  label: string;
  tone?: Tone;
  testId?: string;
}) {
  const palette = toneStyles[tone];
  return (
    <div
      testId={testId}
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 6,
        paddingTop: 4,
        paddingBottom: 4,
        paddingLeft: 9,
        paddingRight: 9,
        borderRadius: 999,
        backgroundColor: palette.bg,
        borderWidth: 1,
        borderColor: palette.border,
      }}
    >
      <div style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: palette.fg }} />
      <text style={textStyle(11, palette.fg, 600)}>{label}</text>
    </div>
  );
}

export function Button({
  label,
  onClick,
  kind = "primary",
  disabled = false,
  testId,
  leading,
  style,
}: {
  label: string;
  onClick?: () => void;
  kind?: "primary" | "secondary" | "ghost" | "danger";
  disabled?: boolean;
  testId?: string;
  leading?: string;
  style?: StyleDesc;
}) {
  const palette: StyleDesc =
    kind === "primary"
      ? {
          background: gradient("#8B7CFF", "#5F4BE8", 110),
          borderWidth: 1,
          borderColor: "#FFFFFF22",
          boxShadow: {
            offsetX: 0,
            offsetY: 8,
            blurRadius: 24,
            spreadRadius: 0,
            color: "#5F4BE866",
          },
        }
      : kind === "danger"
        ? {
            backgroundColor: colors.roseSoft,
            borderWidth: 1,
            borderColor: "#FF7A9044",
          }
        : kind === "secondary"
          ? {
              backgroundColor: "#FFFFFF0D",
              borderWidth: 1,
              borderColor: colors.borderStrong,
            }
          : { backgroundColor: "#00000000" };
  const foreground = kind === "primary" ? "#FFFFFF" : kind === "danger" ? "#FFB5C1" : colors.text;
  return (
    <div
      testId={testId}
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-label={label}
      aria-description={disabled ? "Disabled" : undefined}
      onClick={() => {
        if (!disabled) onClick?.();
      }}
      onKeyDown={(event) => {
        if (disabled) return;
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick?.();
        }
      }}
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        gap: 8,
        height: 36,
        paddingLeft: 16,
        paddingRight: 16,
        borderRadius: 11,
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.45 : 1,
        ...palette,
        hover: disabled
          ? undefined
          : { backgroundColor: kind === "primary" ? "#9A8CFF" : "#FFFFFF16" },
        active: disabled
          ? undefined
          : { backgroundColor: kind === "primary" ? "#7B6BF0" : "#FFFFFF20" },
        focusVisible: focusRing,
        ...style,
      }}
    >
      {leading ? <text style={textStyle(13, foreground, 700)}>{leading}</text> : null}
      <text style={textStyle(13, foreground, 600)}>{label}</text>
    </div>
  );
}

export function IconButton({
  glyph,
  label,
  onClick,
  active = false,
  testId,
}: {
  glyph: string;
  label: string;
  onClick?: () => void;
  active?: boolean;
  testId?: string;
}) {
  return (
    <div
      testId={testId}
      role="button"
      tabIndex={0}
      aria-label={label}
      aria-selected={active}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onClick?.();
        }
      }}
      style={{
        width: 34,
        height: 34,
        borderRadius: 10,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: active ? colors.accentSoft : "#FFFFFF0A",
        borderWidth: 1,
        borderColor: active ? "#8B7CFF55" : colors.border,
        cursor: "pointer",
        hover: { backgroundColor: active ? "#8B7CFF33" : "#FFFFFF14" },
        focusVisible: focusRing,
      }}
    >
      <text style={textStyle(14, active ? "#D6D0FF" : colors.muted, 700)}>{glyph}</text>
    </div>
  );
}

export function Toggle({
  label,
  checked,
  onChange,
  description,
}: {
  label: string;
  checked: boolean;
  onChange: (next: boolean) => void;
  description?: string;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 14 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 3, flexGrow: 1, minWidth: 0 }}>
        <text style={textStyle(13, colors.text, 600)}>{label}</text>
        {description ? <text style={textStyle(11, colors.muted)}>{description}</text> : null}
      </div>
      <div
        role="switch"
        tabIndex={0}
        aria-label={label}
        aria-selected={checked}
        onClick={() => onChange(!checked)}
        onKeyDown={(event) => {
          if (event.key === "enter" || event.key === "space" || event.key === " ") {
            event.preventDefault();
            onChange(!checked);
          }
        }}
        style={{
          width: 44,
          height: 24,
          borderRadius: 12,
          padding: 3,
          backgroundColor: checked ? colors.accent : "#FFFFFF18",
          cursor: "pointer",
          display: "flex",
          flexDirection: "row",
          justifyContent: checked ? "flex-end" : "flex-start",
          focusVisible: focusRing,
        }}
      >
        <div style={{ width: 18, height: 18, borderRadius: 9, backgroundColor: "#FFFFFF" }} />
      </div>
    </div>
  );
}

export function Tabs({
  tabs,
  value,
  onChange,
}: {
  tabs: Array<{ id: string; label: string }>;
  value: string;
  onChange: (id: string) => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        gap: 4,
        padding: 4,
        borderRadius: 12,
        backgroundColor: "#00000033",
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      {tabs.map((tab) => {
        const active = tab.id === value;
        return (
          <div
            key={tab.id}
            role="tab"
            tabIndex={0}
            aria-label={tab.label}
            aria-selected={active}
            onClick={() => onChange(tab.id)}
            onKeyDown={(event) => {
              if (event.key === "enter" || event.key === "space" || event.key === " ") {
                event.preventDefault();
                onChange(tab.id);
              }
            }}
            style={{
              paddingTop: 7,
              paddingBottom: 7,
              paddingLeft: 12,
              paddingRight: 12,
              borderRadius: 9,
              backgroundColor: active ? colors.surfaceRaised : "#00000000",
              borderWidth: active ? 1 : 0,
              borderColor: colors.borderStrong,
              cursor: "pointer",
              focusVisible: focusRing,
            }}
          >
            <text style={textStyle(12, active ? colors.text : colors.muted, active ? 700 : 500)}>
              {tab.label}
            </text>
          </div>
        );
      })}
    </div>
  );
}

export function ProgressBar({ percent, tone = colors.accent }: { percent: number; tone?: string }) {
  const width = Math.max(0, Math.min(100, percent));
  return (
    <div style={{ height: 6, borderRadius: 3, backgroundColor: "#FFFFFF12", overflow: "hidden" }}>
      <div style={{ height: 6, borderRadius: 3, width: `${width}%`, backgroundColor: tone }} />
    </div>
  );
}

export function SummaryTile({
  label,
  value,
  hint,
  tone = colors.text,
}: {
  label: string;
  value: string;
  hint?: string;
  tone?: string;
}) {
  return (
    <div
      style={{
        ...card,
        padding: 14,
        display: "flex",
        flexDirection: "column",
        gap: 6,
        flexGrow: 1,
        minWidth: 0,
      }}
    >
      <text style={{ ...textStyle(10, colors.faint, 700) }}>{label.toUpperCase()}</text>
      <text style={textStyle(24, tone, 700)}>{value}</text>
      {hint ? <text style={textStyle(11, colors.muted)}>{hint}</text> : null}
    </div>
  );
}

export function CaptionRow({
  segment,
  onSimplify,
  locale,
  compact = false,
  captionSize,
}: {
  segment: Segment;
  onSimplify?: (id: string) => void;
  locale: string;
  compact?: boolean;
  captionSize?: number;
}) {
  const rtl = locale === "ar";
  const statusTone =
    segment.status === "clean" ? "mint" : segment.status === "flagged" ? "amber" : "neutral";
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`${segment.text}. ${segment.simplified ?? ""}`}
      aria-description="Simplify this sentence"
      onClick={() => onSimplify?.(segment.id)}
      onKeyDown={(event) => {
        if (event.key === "enter" || event.key === "space" || event.key === " ") {
          event.preventDefault();
          onSimplify?.(segment.id);
        }
      }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: compact ? 4 : 8,
        padding: compact ? 10 : 14,
        borderRadius: 12,
        backgroundColor: "#FFFFFF07",
        borderWidth: 1,
        borderColor: segment.status === "flagged" ? "#FFC46B33" : colors.border,
        cursor: onSimplify ? "pointer" : "default",
        hover: onSimplify ? { backgroundColor: "#FFFFFF0D", borderColor: "#8B7CFF44" } : undefined,
        focusVisible: focusRing,
      }}
    >
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
        <text
          style={{
            ...textStyle(compact ? 11 : 12, colors.faint, 600),
            fontFamily: "Noto Sans Mono",
          }}
        >
          {formatTime(segment.startMs)}
        </text>
        <div style={{ flexGrow: 1 }} />
        {segment.detectedLanguage === "en" ? <Badge label="EN" tone="info" /> : null}
        <Badge
          label={
            segment.status === "clean"
              ? "clean"
              : segment.status === "flagged"
                ? "flagged"
                : "verbatim"
          }
          tone={statusTone}
        />
      </div>
      <text
        style={{
          ...textStyle(compact ? 13 : 15, colors.text, 500),
          lineHeight: compact ? 18 : 22,
          textAlign: rtl ? "right" : "left",
        }}
      >
        {segment.text}
      </text>
      {segment.simplified ? (
        <text
          style={{
            ...textStyle(compact ? 14 : (captionSize ?? 18), "#E4DFFF", 600),
            lineHeight: compact ? 20 : 26,
            textAlign: rtl ? "right" : "left",
          }}
        >
          {segment.simplified}
        </text>
      ) : (
        <text style={textStyle(12, colors.faint)}>Tap to simplify this sentence.</text>
      )}
      {segment.flags.length > 0 ? (
        <div style={{ display: "flex", flexDirection: "row", gap: 6, flexWrap: "wrap" }}>
          {segment.flags.slice(0, 3).map((flag) => (
            <Badge key={flag} label={flag} tone="amber" />
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function formatTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(total / 60)).padStart(2, "0")}:${String(total % 60).padStart(2, "0")}`;
}

export function Divider() {
  return <div style={{ width: "100%", height: 1, backgroundColor: colors.border }} />;
}

export function Slider({
  label,
  value,
  min,
  max,
  onChange,
  unit,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  unit?: string;
}) {
  const percent = ((value - min) / Math.max(1, max - min)) * 100;
  const step = (delta: number) => onChange(Math.max(min, Math.min(max, value + delta)));
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center" }}>
        <text style={{ ...textStyle(13, colors.text, 600), flexGrow: 1 }}>{label}</text>
        <text style={textStyle(12, colors.muted)}>
          {value}
          {unit ?? ""}
        </text>
      </div>
      <div
        role="slider"
        tabIndex={0}
        aria-label={label}
        aria-valuetext={`${value}${unit ?? ""}`}
        onKeyDown={(event) => {
          if (event.key === "arrow-left") {
            event.preventDefault();
            step(-1);
          } else if (event.key === "arrow-right") {
            event.preventDefault();
            step(1);
          }
        }}
        style={{
          height: 22,
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: 10,
          focusVisible: focusRing,
          borderRadius: 11,
        }}
      >
        <div
          onClick={() => step(-1)}
          style={{
            width: 22,
            height: 22,
            borderRadius: 11,
            backgroundColor: "#FFFFFF0D",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
          }}
        >
          <text style={textStyle(12, colors.muted, 700)}>−</text>
        </div>
        <div
          style={{
            flexGrow: 1,
            height: 5,
            borderRadius: 3,
            backgroundColor: "#FFFFFF12",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${percent}%`,
              height: 5,
              borderRadius: 3,
              backgroundColor: colors.accent,
            }}
          />
        </div>
        <div
          onClick={() => step(1)}
          style={{
            width: 22,
            height: 22,
            borderRadius: 11,
            backgroundColor: "#FFFFFF0D",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
          }}
        >
          <text style={textStyle(12, colors.muted, 700)}>+</text>
        </div>
      </div>
    </div>
  );
}

export function Toast({ message, tone = "info" }: { message: string; tone?: Tone }) {
  const palette = toneStyles[tone];
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 8,
        paddingTop: 8,
        paddingBottom: 8,
        paddingLeft: 12,
        paddingRight: 12,
        borderRadius: 10,
        backgroundColor: palette.bg,
        borderWidth: 1,
        borderColor: palette.border,
      }}
    >
      <text style={textStyle(12, palette.fg, 600)}>{message}</text>
    </div>
  );
}
