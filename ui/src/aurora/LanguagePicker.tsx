import { useMemo, useState } from "react";
import { Badge, SectionHeader } from "../aurora/components";
import { colors, focusRing, textStyle } from "../aurora/theme";
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
  height?: number;
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
          eyebrow={t(locale, "LANGUAGE")}
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
          height: 38,
          paddingLeft: 12,
          paddingRight: 12,
          borderRadius: 11,
          backgroundColor: "#00000033",
          borderWidth: 1,
          borderColor: colors.border,
        }}
      >
        <text style={textStyle(12, colors.faint, 700)}>⌕</text>
        <input
          value={query}
          placeholder={t(locale, "Search languages")}
          onChange={(event) => setQuery(event.value ?? "")}
          style={{
            flexGrow: 1,
            minWidth: 0,
            fontSize: 13,
            color: colors.text,
            backgroundColor: "#00000000",
            borderWidth: 0,
            focusVisible: { outlineWidth: 0 },
          }}
        />
      </div>
      <div
        style={{
          height,
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
              aria-label={language.name}
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
                paddingTop: 9,
                paddingBottom: 9,
                paddingLeft: 12,
                paddingRight: 12,
                borderRadius: 10,
                backgroundColor: active ? colors.accentSoft : "#FFFFFF06",
                borderWidth: 1,
                borderColor: active ? "#8B7CFF55" : "#FFFFFF0A",
                cursor: "pointer",
                hover: { backgroundColor: active ? "#8B7CFF28" : "#FFFFFF0D" },
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
                <text style={textStyle(13, colors.text, active ? 700 : 500)}>{language.name}</text>
                <text style={textStyle(10, colors.faint)}>{language.code}</text>
              </div>
              <Badge
                label={statusLabel(language.asrStatus)}
                tone={
                  language.asrStatus === "ok"
                    ? "mint"
                    : language.asrStatus === "degraded"
                      ? "amber"
                      : "neutral"
                }
              />
            </div>
          );
        })}
        {filtered.length === 0 ? (
          <div style={{ padding: 16, display: "flex", justifyContent: "center" }}>
            <text style={textStyle(12, colors.muted)}>No languages match “{query}”.</text>
          </div>
        ) : null}
      </div>
    </div>
  );
}
