import { useState } from "react";
import { Badge, Button, Divider, SectionHeader, Surface, Toast } from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, gradient, textStyle } from "../aurora/theme";
import { languageByCode } from "../i18n/languages";
import { t } from "../i18n/locales";

export function Onboarding({
  language,
  locale,
  onLanguage,
  onFinish,
  onOpenLicenses,
}: {
  language: string;
  locale: string;
  onLanguage: (code: string) => void;
  onFinish: () => void;
  onOpenLicenses: () => void;
}) {
  const [beat, setBeat] = useState(Number(process.env.LL_ONBOARDING_BEAT ?? 0));
  const [toast, setToast] = useState<string | null>(null);
  const selected = languageByCode(language);

  const steps = [
    t(locale, "Welcome to Live Linguist"),
    t(locale, "Choose your language"),
    t(locale, "On-device models"),
  ];

  return (
    <div style={{ display: "flex", flexDirection: "row", gap: 28, padding: 36, height: "100%" }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 22, width: 360, flexShrink: 0 }}>
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
          <div
            style={{
              width: 42,
              height: 42,
              borderRadius: 13,
              background: gradient("#8B7CFF", "#2ED3B7", 135),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <text style={textStyle(15, "#0B0B12", 800)}>LL</text>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            <text style={textStyle(14, colors.text, 700)}>Live Linguist</text>
            <text style={textStyle(10, colors.faint, 600)}>ON-DEVICE · WINDOWS 11</text>
          </div>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {steps.map((step, index) => (
            <div
              key={step}
              style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}
            >
              <div
                style={{
                  width: 22,
                  height: 22,
                  borderRadius: 11,
                  backgroundColor: index <= beat ? colors.accent : "#FFFFFF10",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <text style={textStyle(10, index <= beat ? "#0B0B12" : colors.muted, 800)}>
                  {index + 1}
                </text>
              </div>
              <text
                style={textStyle(
                  12,
                  index === beat ? colors.text : colors.muted,
                  index === beat ? 700 : 500,
                )}
              >
                {step}
              </text>
            </div>
          ))}
        </div>

        <div style={{ flexGrow: 1 }} />

        <Surface style={{ padding: 16, display: "flex", flexDirection: "column", gap: 10 }}>
          <text style={textStyle(11, colors.mint, 700)}>PRIVACY BY CONSTRUCTION</text>
          <text style={textStyle(12, colors.muted)}>
            {t(
              locale,
              "Audio is processed entirely on this device. Nothing is sent to any server. Transcripts are saved only when you export them.",
            )}
          </text>
          <Divider />
          <div
            role="button"
            tabIndex={0}
            aria-label={t(locale, "Licenses")}
            onClick={onOpenLicenses}
            style={{ display: "flex", flexDirection: "row", cursor: "pointer" }}
          >
            <text style={textStyle(11, colors.faint)}>{t(locale, "Licenses")} →</text>
          </div>
        </Surface>
      </div>

      <div style={{ flexGrow: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 18 }}>
        {beat === 0 ? (
          <Surface
            style={{ padding: 30, display: "flex", flexDirection: "column", gap: 18, flexGrow: 1 }}
          >
            <Badge label={t(locale, "LIVE LINGUIST · ON-DEVICE")} tone="accent" />
            <text style={{ ...textStyle(38, colors.text, 800), lineHeight: 44 }}>
              {t(locale, "Welcome to Live Linguist")}
            </text>
            <text style={{ ...textStyle(15, colors.muted), lineHeight: 24 }}>
              {t(
                locale,
                "Live Linguist speaks %lld languages — captions, simplification, and the app itself.",
                29,
              )}
            </text>
            <div style={{ display: "flex", flexDirection: "row", gap: 12, flexWrap: "wrap" }}>
              <Badge label={t(locale, "Offline — audio never leaves this device")} tone="mint" />
              <Badge label={t(locale, "Same language, not translation")} tone="info" />
              <Badge label={t(locale, "29 locales")} tone="neutral" />
            </div>
            <Surface
              style={{
                padding: 16,
                backgroundColor: "#00000033",
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
                <text style={{ ...textStyle(10, colors.faint, 700) }}>LIVE CAPTION SAMPLE</text>
                <div style={{ flexGrow: 1 }} />
                <Badge label="FALC" tone="accent" />
                <Badge label="clean" tone="mint" />
              </div>
              <text style={textStyle(13, colors.muted)}>
                Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera.
              </text>
              <text style={textStyle(18, "#E4DFFF", 600)}>
                Viens nous voir bientôt. Viens quand tu peux.
              </text>
            </Surface>
            <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
              <MiniFeature title="Two tiers" detail="verbatim partial + simplified final" />
              <MiniFeature title="29 locales" detail="official easy-language registers" />
              <MiniFeature title="On-device" detail="no cloud, no translation" />
            </div>
            <div style={{ flexGrow: 1 }} />
            <div style={{ display: "flex", flexDirection: "row", gap: 10, alignItems: "center" }}>
              <Button
                label={t(locale, "Continue")}
                onClick={() => setBeat(1)}
                style={{ width: 140 }}
              />
              <Button
                label={t(locale, "Continue in demo mode")}
                kind="ghost"
                onClick={() => onFinish()}
              />
            </div>
          </Surface>
        ) : null}

        {beat === 1 ? (
          <Surface
            style={{ padding: 26, display: "flex", flexDirection: "column", gap: 16, flexGrow: 1 }}
          >
            <LanguagePicker
              selected={language}
              onSelect={onLanguage}
              locale={locale}
              height={330}
            />
            <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
              <Button label={t(locale, "Back")} kind="secondary" onClick={() => setBeat(0)} />
              <Button label={t(locale, "Continue")} onClick={() => setBeat(2)} />
            </div>
          </Surface>
        ) : null}

        {beat === 2 ? (
          <Surface
            style={{ padding: 26, display: "flex", flexDirection: "column", gap: 16, flexGrow: 1 }}
          >
            <SectionHeader
              eyebrow={t(locale, "MODELS")}
              title={t(locale, "On-device models")}
              subtitle={t(
                locale,
                "Live captions need a one-time model download (about %@). Everything runs offline afterward.",
                "1.0 GB",
              )}
            />
            <div style={{ display: "flex", flexDirection: "row", gap: 12 }}>
              <ModelChip
                title="Qwen3 0.6B"
                detail="EasyLanguage · Q4_K_M"
                size="~400 MB"
                tone="accent"
              />
              <ModelChip
                title="Nemotron ASR"
                detail="streaming · int8"
                size="~600 MB"
                tone="mint"
              />
              <ModelChip title="Silero VAD" detail="voice activity" size="~2 MB" tone="neutral" />
            </div>
            <Surface
              style={{
                padding: 14,
                backgroundColor: "#00000033",
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              <text style={textStyle(12, colors.text, 600)}>
                {selected.name} ·{" "}
                {selected.asrStatus === "simplify_only"
                  ? "simplify-only preview"
                  : "live captioning"}
              </text>
              <text style={textStyle(12, colors.muted)}>
                {selected.asrStatus === "simplify_only"
                  ? t(
                      locale,
                      "Live listening is not available for this language yet. Paste or type text to simplify it.",
                    )
                  : t(locale, "Ready to caption")}
              </text>
              <text style={textStyle(11, colors.faint)}>
                {t(locale, "The app contacts Hugging Face only when you start a model download.")}
              </text>
            </Surface>
            {toast ? <Toast message={toast} tone="amber" /> : null}
            <div style={{ flexGrow: 1 }} />
            <div style={{ display: "flex", flexDirection: "row", gap: 10 }}>
              <Button label={t(locale, "Back")} kind="secondary" onClick={() => setBeat(1)} />
              <Button
                label={t(locale, "Download models (~%@)", "1.0 GB")}
                onClick={() =>
                  setToast(
                    "Model catalog is not published in this build. The downloader is implemented; hashes are required before a download can start.",
                  )
                }
              />
              <Button
                label={t(locale, "Continue in demo mode")}
                kind="secondary"
                onClick={onFinish}
              />
            </div>
          </Surface>
        ) : null}
      </div>
    </div>
  );
}

function ModelChip({
  title,
  detail,
  size,
  tone,
}: {
  title: string;
  detail: string;
  size: string;
  tone: "accent" | "mint" | "neutral";
}) {
  const color = tone === "accent" ? colors.accent : tone === "mint" ? colors.mint : colors.muted;
  return (
    <div
      style={{
        flexGrow: 1,
        minWidth: 0,
        padding: 12,
        borderRadius: 12,
        backgroundColor: "#FFFFFF07",
        borderWidth: 1,
        borderColor: colors.border,
        display: "flex",
        flexDirection: "column",
        gap: 5,
      }}
    >
      <text style={textStyle(13, colors.text, 700)}>{title}</text>
      <text style={textStyle(10, color, 600)}>{detail}</text>
      <text style={textStyle(11, colors.faint)}>{size}</text>
    </div>
  );
}

function MiniFeature({ title, detail }: { title: string; detail: string }) {
  return (
    <div
      style={{
        flexGrow: 1,
        minWidth: 0,
        padding: 10,
        borderRadius: 10,
        backgroundColor: "#FFFFFF06",
        borderWidth: 1,
        borderColor: colors.border,
        display: "flex",
        flexDirection: "column",
        gap: 3,
      }}
    >
      <text style={textStyle(12, colors.text, 700)}>{title}</text>
      <text style={textStyle(10, colors.faint)}>{detail}</text>
    </div>
  );
}
