import type { StyleDesc } from "@gpuix/react";
import type { ReactNode } from "react";
import { captionAccessibleLabel } from "../captionKeys";
import { directionFor } from "../i18n/direction";
import { t } from "../i18n/locales";
import type { Segment } from "../store/sessionStore";
import { card, colors, focusRing, fonts, glass, gradient, textStyle, type } from "./theme";

/**
 * A barely-there backdrop: flat near-black plus two very low-alpha washes.
 * Quiet by design; `active` only lifts them a few points while listening.
 */
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
        backgroundColor: colors.bgDeep,
      }}
    >
      <div
        style={{
          position: "absolute",
          top: -300,
          left: -220,
          width: 780,
          height: 780,
          borderRadius: 390,
          background: gradient(active ? "#7A6BFF1A" : "#7A6BFF12", "#00000000", 90),
        }}
      />
      <div
        style={{
          position: "absolute",
          bottom: -380,
          right: -260,
          width: 900,
          height: 900,
          borderRadius: 450,
          background: gradient(active ? "#2ED3B714" : "#2ED3B70C", "#00000000", 300),
        }}
      />
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          height: 1,
          backgroundColor: "#FFFFFF10",
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

/** Flat product mark: one accent tile, no gradient. */
export function BrandMark({ size = 34 }: { size?: number }) {
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: Math.round(size * 0.28),
        backgroundColor: colors.accent,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0,
      }}
    >
      <text style={textStyle(Math.round(size * 0.52), "#0A0A0D", 800)}>L</text>
    </div>
  );
}

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
          <text style={{ ...textStyle(type.meta, colors.accent, 700) }}>
            {eyebrow.toUpperCase()}
          </text>
        ) : null}
        <text style={{ ...textStyle(type.title, colors.text, 600), lineHeight: 26 }}>{title}</text>
        {subtitle ? (
          <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 20 }}>{subtitle}</text>
        ) : null}
      </div>
      {trailing}
    </div>
  );
}

/** Top bar for a full-screen page: back affordance, title, trailing controls. */
export function PageHeader({
  backLabel,
  eyebrow,
  title,
  subtitle,
  onBack,
  trailing,
}: {
  backLabel?: string;
  eyebrow?: string;
  title: string;
  subtitle?: string;
  onBack?: () => void;
  trailing?: ReactNode;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 14 }}>
      {onBack ? <Button label={backLabel ?? "Back"} kind="secondary" onClick={onBack} /> : null}
      <div style={{ display: "flex", flexDirection: "column", gap: 3, flexGrow: 1, minWidth: 0 }}>
        {eyebrow ? (
          <text style={{ ...textStyle(type.meta, colors.accent, 700) }}>
            {eyebrow.toUpperCase()}
          </text>
        ) : null}
        <text style={{ ...textStyle(type.page, colors.text, 600), lineHeight: 34 }}>{title}</text>
        {subtitle ? <text style={textStyle(type.body, colors.muted)}>{subtitle}</text> : null}
      </div>
      {trailing}
    </div>
  );
}

type Tone = "neutral" | "accent" | "mint" | "amber" | "rose" | "info";

/** Primary button fills; white labels need >= 4.5:1 on every one (contrast test). */
export const PRIMARY_FILL = {
  from: "#6B58E0",
  to: "#5F4CD4",
  hover: "#6350D8",
  active: "#5745C6",
} as const;

export const toneStyles: Record<Tone, { bg: string; fg: string; border: string }> = {
  neutral: { bg: "#FFFFFF0D", fg: colors.muted, border: colors.border },
  accent: { bg: colors.accentSoft, fg: "#D8D2FF", border: "#AFA2FF44" },
  mint: { bg: colors.mintSoft, fg: "#ABF5E4", border: "#5FE3C044" },
  amber: { bg: colors.amberSoft, fg: colors.amber, border: "#F5C06A44" },
  rose: { bg: colors.roseSoft, fg: "#FFB9C4", border: "#FF8A9B44" },
  info: { bg: "#4EA8FF1F", fg: "#A6D4FF", border: "#4EA8FF44" },
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
        alignSelf: "flex-start",
        gap: 6,
        paddingTop: 3,
        paddingBottom: 3,
        paddingLeft: 8,
        paddingRight: 8,
        borderRadius: 6,
        backgroundColor: palette.bg,
        borderWidth: 1,
        borderColor: palette.border,
      }}
    >
      <div style={{ width: 5, height: 5, borderRadius: 3, backgroundColor: palette.fg }} />
      <text style={{ ...textStyle(type.meta, palette.fg, 600), lineHeight: 16 }}>{label}</text>
    </div>
  );
}

export function StatusDot({
  tone = "mint",
  label,
}: {
  tone?: "mint" | "amber" | "rose" | "neutral" | "accent";
  label: string;
}) {
  const color =
    tone === "mint"
      ? colors.mint
      : tone === "amber"
        ? colors.amber
        : tone === "rose"
          ? colors.rose
          : tone === "accent"
            ? colors.accent
            : colors.faint;
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 6 }}>
      <div style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: color }} />
      <text style={textStyle(type.meta, colors.muted, 600)}>{label}</text>
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
  onMouseDown,
}: {
  label: string;
  onClick?: () => void;
  kind?: "primary" | "secondary" | "ghost" | "danger";
  disabled?: boolean;
  testId?: string;
  leading?: string;
  style?: StyleDesc;
  onMouseDown?: () => void;
}) {
  const palette: StyleDesc =
    kind === "primary"
      ? {
          backgroundColor: PRIMARY_FILL.from,
          borderWidth: 1,
          borderColor: "#FFFFFF1F",
        }
      : kind === "danger"
        ? {
            backgroundColor: colors.roseSoft,
            borderWidth: 1,
            borderColor: "#FF8A9B44",
          }
        : kind === "secondary"
          ? {
              backgroundColor: "#FFFFFF0A",
              borderWidth: 1,
              borderColor: colors.borderStrong,
            }
          : { backgroundColor: "#00000000" };
  const foreground = kind === "primary" ? "#FFFFFF" : kind === "danger" ? "#FFB9C4" : colors.text;
  return (
    <div
      testId={testId}
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-label={label}
      aria-disabled={disabled || undefined}
      onClick={() => {
        if (!disabled) onClick?.();
      }}
      onMouseDown={onMouseDown}
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
        height: 38,
        paddingLeft: 14,
        paddingRight: 14,
        borderRadius: 8,
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        ...palette,
        hover: disabled
          ? undefined
          : { backgroundColor: kind === "primary" ? PRIMARY_FILL.hover : "#FFFFFF14" },
        active: disabled
          ? undefined
          : { backgroundColor: kind === "primary" ? PRIMARY_FILL.active : "#FFFFFF1C" },
        focusVisible: focusRing,
        ...style,
      }}
    >
      {leading ? <text style={textStyle(type.label, foreground, 700)}>{leading}</text> : null}
      <text style={{ ...textStyle(type.body, foreground, 600), whiteSpace: "nowrap" }}>
        {label}
      </text>
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
        borderRadius: 8,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: active ? colors.accentSoft : "#FFFFFF0A",
        borderWidth: 1,
        borderColor: active ? "#AFA2FF55" : colors.border,
        cursor: "pointer",
        hover: { backgroundColor: active ? "#AFA2FF33" : "#FFFFFF14" },
        focusVisible: focusRing,
      }}
    >
      <text style={textStyle(type.body, active ? "#D8D2FF" : colors.muted, 700)}>{glyph}</text>
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
        <text style={textStyle(type.body, colors.text, 600)}>{label}</text>
        {description ? (
          <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
            {description}
          </text>
        ) : null}
      </div>
      <div
        role="switch"
        tabIndex={0}
        aria-label={label}
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        onKeyDown={(event) => {
          if (event.key === "enter" || event.key === "space" || event.key === " ") {
            event.preventDefault();
            onChange(!checked);
          }
        }}
        style={{
          width: 40,
          height: 22,
          borderRadius: 11,
          padding: 3,
          backgroundColor: checked ? colors.accent : "#FFFFFF1A",
          cursor: "pointer",
          display: "flex",
          flexDirection: "row",
          justifyContent: checked ? "flex-end" : "flex-start",
          focusVisible: focusRing,
        }}
      >
        <div style={{ width: 16, height: 16, borderRadius: 8, backgroundColor: "#FFFFFF" }} />
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
      role="tablist"
      style={{
        display: "flex",
        flexDirection: "row",
        alignSelf: "flex-start",
        gap: 4,
        padding: 3,
        borderRadius: 10,
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
              paddingTop: 6,
              paddingBottom: 6,
              paddingLeft: 12,
              paddingRight: 12,
              borderRadius: 7,
              backgroundColor: active ? colors.surfaceRaised : "#00000000",
              borderWidth: active ? 1 : 1,
              borderColor: active ? colors.borderStrong : "#00000000",
              cursor: "pointer",
              focusVisible: focusRing,
            }}
          >
            <text
              style={textStyle(type.label, active ? colors.text : colors.muted, active ? 600 : 500)}
            >
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
    <div style={{ height: 4, borderRadius: 2, backgroundColor: "#FFFFFF12", overflow: "hidden" }}>
      <div style={{ height: 4, borderRadius: 2, width: `${width}%`, backgroundColor: tone }} />
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
        padding: 16,
        display: "flex",
        flexDirection: "column",
        gap: 6,
        flexGrow: 1,
        minWidth: 0,
      }}
    >
      <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>{label.toUpperCase()}</text>
      <text style={{ ...textStyle(26, tone, 600), lineHeight: 30 }}>{value}</text>
      {hint ? <text style={textStyle(type.meta, colors.muted)}>{hint}</text> : null}
    </div>
  );
}

export function CaptionRow({
  segment,
  onSimplify,
  onKey,
  locale,
  compact = false,
  captionSize,
  showTier1 = true,
  simplifying = false,
  autoFocus = false,
}: {
  segment: Segment;
  onSimplify?: (id: string) => void;
  /** List-level keyboard handling (Up/Down/Enter); returns true if handled. */
  onKey?: (key: string) => boolean;
  locale: string;
  compact?: boolean;
  captionSize?: number;
  showTier1?: boolean;
  simplifying?: boolean;
  autoFocus?: boolean;
}) {
  const rtl = directionFor(locale) === "rtl";
  return (
    <div
      role="button"
      tabIndex={0}
      // biome-ignore lint/a11y/noAutofocus: set only after the user's own Up/Down or Read latest; GPUIX has no imperative focus().
      autoFocus={autoFocus}
      aria-label={captionAccessibleLabel(segment.text, segment.simplified, simplifying)}
      aria-description="Enter or Space simplifies this sentence; Up and Down move between sentences"
      onClick={() => onSimplify?.(segment.id)}
      onKeyDown={(event) => {
        const key = event.key ?? "";
        if (onKey?.(key)) {
          event.preventDefault();
          return;
        }
        if (key === "enter" || key === "space" || key === " ") {
          event.preventDefault();
          onSimplify?.(segment.id);
        }
      }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: compact ? 4 : 8,
        padding: compact ? 10 : 14,
        borderRadius: 10,
        backgroundColor: "#FFFFFF07",
        borderWidth: 1,
        borderColor: segment.status === "flagged" ? "#F5C06A33" : colors.border,
        cursor: onSimplify ? "pointer" : "default",
        hover: onSimplify ? { backgroundColor: "#FFFFFF0D", borderColor: "#AFA2FF44" } : undefined,
        focusVisible: focusRing,
      }}
    >
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
        <text
          style={{
            ...textStyle(type.meta, colors.faint, 500),
            fontFamily: fonts.mono,
          }}
        >
          {formatTime(segment.startMs)}
        </text>
        <div style={{ flexGrow: 1 }} />
        {segment.detectedLanguage === "en" ? (
          <Badge label={t(locale, "English")} tone="info" />
        ) : null}
        {segment.status === "flagged" ? (
          <Badge label={t(locale, "Check wording")} tone="amber" />
        ) : null}
      </div>
      {showTier1 ? (
        <text
          style={{
            ...textStyle(compact ? type.label : 15, colors.muted, 500),
            lineHeight: compact ? 19 : 22,
            textAlign: rtl ? "right" : "left",
          }}
        >
          {segment.text}
        </text>
      ) : null}
      {segment.simplified ? (
        <text
          style={{
            ...textStyle(compact ? 15 : (captionSize ?? 18), colors.text, 600),
            lineHeight: compact ? 22 : Math.round((captionSize ?? 18) * 1.35),
            textAlign: rtl ? "right" : "left",
          }}
        >
          {segment.simplified}
        </text>
      ) : (
        <text style={textStyle(type.meta, colors.faint)}>
          {simplifying
            ? t(locale, "Simplifying…")
            : t(locale, "Click or press Enter to simplify this sentence.")}
        </text>
      )}
      {segment.flags.length > 0 ? (
        <div style={{ display: "flex", flexDirection: "row", gap: 6, flexWrap: "wrap" }}>
          {segment.flags.slice(0, 2).map((flag) => (
            <Badge key={flag} label={flagLabel(flag)} tone="neutral" />
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

/** `passive_voice` -> `Passive voice`, so validator codes read as words. */
export function flagLabel(flag: string): string {
  const words = flag.replace(/[_-]+/g, " ").trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
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
  const clamp = (next: number) => Math.max(min, Math.min(max, next));
  const step = (delta: number) => onChange(clamp(value + delta));
  const key = (event: { key?: string; preventDefault: () => void }) => {
    const name = (event.key ?? "").toLowerCase();
    if (name === "arrow-left" || name === "arrow-down") {
      event.preventDefault();
      step(-1);
    } else if (name === "arrow-right" || name === "arrow-up") {
      event.preventDefault();
      step(1);
    } else if (name === "pageup") {
      event.preventDefault();
      step(5);
    } else if (name === "pagedown") {
      event.preventDefault();
      step(-5);
    } else if (name === "home") {
      event.preventDefault();
      onChange(min);
    } else if (name === "end") {
      event.preventDefault();
      onChange(max);
    }
  };
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center" }}>
        <text style={{ ...textStyle(type.body, colors.text, 600), flexGrow: 1 }}>{label}</text>
        <text style={textStyle(type.label, colors.muted, 600)}>{`${value}${unit ?? ""}`}</text>
      </div>
      <div
        role="slider"
        tabIndex={0}
        aria-label={label}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        aria-valuetext={`${value}${unit ?? ""}`}
        onKeyDown={key}
        style={{
          height: 28,
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: 12,
          focusVisible: focusRing,
          borderRadius: 8,
        }}
      >
        <div
          role="button"
          tabIndex={0}
          aria-label={`${label} minus`}
          onClick={() => step(-1)}
          onKeyDown={key}
          style={{
            width: 30,
            height: 30,
            borderRadius: 8,
            backgroundColor: "#FFFFFF0D",
            borderWidth: 1,
            borderColor: colors.border,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
            hover: { backgroundColor: "#FFFFFF16" },
            focusVisible: focusRing,
          }}
        >
          <text style={textStyle(type.body, colors.muted, 700)}>−</text>
        </div>
        <div
          style={{
            flexGrow: 1,
            height: 6,
            borderRadius: 3,
            backgroundColor: "#FFFFFF12",
          }}
        >
          <div
            style={{
              width: `${percent}%`,
              height: 6,
              borderRadius: 3,
              backgroundColor: colors.accent,
              display: "flex",
              flexDirection: "row",
              alignItems: "center",
              justifyContent: "flex-end",
            }}
          >
            <div
              style={{
                width: 16,
                height: 16,
                borderRadius: 8,
                backgroundColor: colors.text,
                borderWidth: 2,
                borderColor: colors.bg,
                flexShrink: 0,
              }}
            />
          </div>
        </div>
        <div
          role="button"
          tabIndex={0}
          aria-label={`${label} plus`}
          onClick={() => step(1)}
          onKeyDown={key}
          style={{
            width: 30,
            height: 30,
            borderRadius: 8,
            backgroundColor: "#FFFFFF0D",
            borderWidth: 1,
            borderColor: colors.border,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
            hover: { backgroundColor: "#FFFFFF16" },
            focusVisible: focusRing,
          }}
        >
          <text style={textStyle(type.body, colors.muted, 700)}>+</text>
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
        paddingTop: 9,
        paddingBottom: 9,
        paddingLeft: 12,
        paddingRight: 12,
        borderRadius: 8,
        backgroundColor: palette.bg,
        borderWidth: 1,
        borderColor: palette.border,
      }}
    >
      <text style={textStyle(type.label, palette.fg, 600)}>{message}</text>
    </div>
  );
}
