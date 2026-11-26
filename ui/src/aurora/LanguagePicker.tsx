import { useMemo, useState } from "react";
import { Badge, SectionHeader } from "../aurora/components";
import { colors, focusRing, textStyle, type } from "../aurora/theme";
import { LANGUAGES, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";

export function LanguagePicker({
  selected,
  onSelect,
  locale,
  height = 330,
  showHeader = true,
}: {
  selected: string;
  onSelect: (code: string) => void;
  locale: string;
  /** A number for a fixed list, or "fill" to take the remaining card height. */
  height?: number | "fill";
  showHeader?: boolean;
}) {
  const [query, setQuery] = useState("");
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return LANGUAGES;
    return LANGUAGES.filter(
      (language) =>
        language.name.toLowerCase().includes(needle) ||
        language.shortName.toLowerCase().includes(needle) ||
        language.code.toLowerCase().includes(needle),
    );
  }, [query]);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12, flexGrow: 1, minHeight: 0 }}>
      {showHeader ? (
        <SectionHeader
          title={t(locale, "Choose your language")}
          subtitle={t(
            locale,
            "Live Linguist speaks %lld languages — captions, simplification, and the app itself.",
            29,
          )}
        />
      ) : null}
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: 8,
          height: 40,
          paddingLeft: 12,
          paddingRight: 12,
          borderRadius: 8,
          backgroundColor: "#00000033",
          borderWidth: 1,
          borderColor: colors.border,
        }}
      >
        <input
          value={query}
          placeholder={t(locale, "Search languages")}
          onChange={(event) => setQuery(event.value ?? "")}
          style={{
            flexGrow: 1,
            minWidth: 0,
            fontSize: type.label,
            color: colors.text,
            backgroundColor: "#00000000",
            borderWidth: 0,
            focusVisible: { outlineWidth: 0 },
          }}
        />
      </div>
      <div
        style={{
          height: height === "fill" ? undefined : height,
          flexGrow: height === "fill" ? 1 : 0,
          minHeight: 0,
          overflow: "scroll",
          scrollbarColor: "#FFFFFF22",
          scrollbarWidth: 6,
          display: "flex",
          flexDirection: "column",
          gap: 6,
          paddingRight: 6,
        }}
      >
        {filtered.map((language) => {
          const active = language.code === selected;
          return (
            <div
              key={language.code}
              role="button"
              tabIndex={0}
              aria-label={`${language.name}. ${statusLabel(language.asrStatus)}`}
              aria-selected={active}
              onClick={() => onSelect(language.code)}
              onKeyDown={(event) => {
                if (event.key === "enter" || event.key === "space" || event.key === " ") {
                  event.preventDefault();
                  onSelect(language.code);
                }
              }}
              style={{
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                gap: 10,
                paddingTop: 10,
                paddingBottom: 10,
                paddingLeft: 12,
                paddingRight: 12,
                borderRadius: 8,
                backgroundColor: active ? colors.accentSoft : "#FFFFFF06",
                borderWidth: 1,
                borderColor: active ? "#AFA2FF66" : "#FFFFFF0F",
                cursor: "pointer",
                hover: { backgroundColor: active ? "#AFA2FF2E" : "#FFFFFF0D" },
                focusVisible: focusRing,
              }}
            >
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 2,
                  flexGrow: 1,
                  minWidth: 0,
                }}
              >
                <text style={textStyle(type.body, colors.text, active ? 600 : 500)}>
                  {language.name}
                </text>
                <text style={textStyle(type.meta, colors.faint)}>{language.code}</text>
              </div>
              {language.asrStatus === "ok" ? null : (
                <Badge
                  label={statusLabel(language.asrStatus)}
                  tone={language.asrStatus === "degraded" ? "amber" : "neutral"}
                />
              )}
            </div>
          );
        })}
        {filtered.length === 0 ? (
          <div style={{ padding: 16, display: "flex", justifyContent: "center" }}>
            <text style={textStyle(type.body, colors.muted)}>No languages match “{query}”.</text>
          </div>
        ) : null}
      </div>
    </div>
  );
}
