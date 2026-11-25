import { useState } from "react";
import {
  Badge,
  BrandMark,
  Button,
  SectionHeader,
  StatusDot,
  Surface,
  Toast,
} from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode, statusDetail, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import { allReady, type ModelEntry, type ModelView, megabytes, percent } from "../store/modelStore";

/**
 * Three beats: welcome, language, models. One column, one card, one primary
 * action per beat; the privacy line stays at the bottom of the card.
 */
export function Onboarding({
  language,
  locale,
  onLanguage,
  onFinish,
  onOpenLicenses,
  models,
  onDownload,
  onCancelDownload,
}: {
  language: string;
  locale: string;
  onLanguage: (code: string) => void;
  onFinish: () => void;
  onOpenLicenses: () => void;
  models: ModelView;
  onDownload: () => void;
  onCancelDownload: () => void;
}) {
  const ready = allReady(models);
  const [beat, setBeat] = useState(Number(process.env.LL_ONBOARDING_BEAT ?? 0));
  const selected = languageByCode(language);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        height: "100%",
        padding: 28,
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: 860,
          height: "100%",
          display: "flex",
          flexDirection: "column",
          gap: 14,
          minHeight: 0,
        }}
      >
        <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
          <BrandMark size={32} />
          <div style={{ display: "flex", flexDirection: "column", gap: 1, flexGrow: 1 }}>
            <text style={textStyle(type.label, colors.text, 600)}>Live Linguist</text>
            <text style={textStyle(type.meta, colors.faint)}>
              {t(locale, "On-device · Windows 11")}
            </text>
          </div>
          <text style={textStyle(type.meta, colors.faint, 600)}>{`${beat + 1} / 3`}</text>
        </div>

        <div style={{ display: "flex", flexDirection: "row", gap: 6 }}>
          {[0, 1, 2].map((step) => (
            <div
              key={step}
              style={{
                flexGrow: 1,
                height: 3,
                borderRadius: 2,
                backgroundColor: step <= beat ? colors.accent : "#FFFFFF14",
              }}
            />
          ))}
        </div>

        <Surface
          style={{
            flexGrow: 1,
            minHeight: 0,
            display: "flex",
            flexDirection: "column",
            gap: 18,
            padding: 28,
          }}
        >
          {beat === 0 ? (
            <div
              style={{
                flexGrow: 1,
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                gap: 20,
              }}
            >
              <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
                <text style={{ ...textStyle(type.hero, colors.text, 600), lineHeight: 40 }}>
                  {t(locale, "Welcome to Live Linguist")}
                </text>
                <text style={{ ...textStyle(15, colors.muted), lineHeight: 23 }}>
                  {t(
                    locale,
                    "Live Linguist speaks %lld languages — captions, simplification, and the app itself.",
                    29,
                  )}
                </text>
                <text style={textStyle(type.meta, colors.faint)}>
                  {t(locale, "On-device · Same language, not translation · 29 locales")}
                </text>
              </div>

              <div
                style={{
                  padding: 16,
                  borderRadius: 10,
                  backgroundColor: "#00000033",
                  borderWidth: 1,
                  borderColor: colors.border,
                  display: "flex",
                  flexDirection: "column",
                  gap: 10,
                }}
              >
                <div
                  style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}
                >
                  <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>
                    {t(locale, "LIVE CAPTION SAMPLE").toUpperCase()}
                  </text>
                  <div style={{ flexGrow: 1 }} />
                  <Badge label="FALC" tone="accent" />
                </div>
                <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 20 }}>
                  Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera.
                </text>
                <text style={{ ...textStyle(20, colors.text, 600), lineHeight: 27 }}>
                  Viens nous voir bientôt. Viens quand tu peux.
                </text>
              </div>
            </div>
          ) : null}

          {beat === 1 ? (
            <>
              <LanguagePicker
                selected={language}
                onSelect={onLanguage}
                locale={locale}
                height="fill"
              />
              <text style={{ ...textStyle(type.meta, colors.faint), lineHeight: 18 }}>
                {t(
                  locale,
                  "Languages marked “%@” take pasted or typed text instead of sound.",
                  statusLabel("simplify_only"),
                )}
              </text>
            </>
          ) : null}

          {beat === 2 ? (
            <div
              style={{
                flexGrow: 1,
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                gap: 18,
              }}
            >
              <SectionHeader
                title={t(locale, "On-device models")}
                subtitle={t(
                  locale,
                  "Live captions need a one-time model download (about %@). Everything runs offline afterward.",
                  "1.08 GB",
                )}
              />
              <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
                {models.models.map((model) => (
                  <ModelProgress key={model.id} model={model} locale={locale} />
                ))}
              </div>
              {ready ? <StatusDot tone="mint" label={t(locale, "Models ready")} /> : null}
              {models.error ? (
                <Toast
                  message={
                    models.error.code === "CANCELLED"
                      ? "Download cancelled. Retry resumes where it stopped."
                      : `Download failed (${models.error.code}): ${models.error.reason}`
                  }
                  tone={models.error.code === "CANCELLED" ? "amber" : "rose"}
                />
              ) : null}
              <div
                style={{
                  padding: 14,
                  borderRadius: 10,
                  backgroundColor: "#00000033",
                  borderWidth: 1,
                  borderColor: colors.border,
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                }}
              >
                <div
                  style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}
                >
                  <text style={textStyle(type.body, colors.text, 600)}>{selected.name}</text>
                  <StatusDot
                    tone={selected.asrStatus === "ok" ? "mint" : "amber"}
                    label={statusLabel(selected.asrStatus)}
                  />
                </div>
                <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
                  {statusDetail(selected.asrStatus)}
                </text>
                <text style={textStyle(type.meta, colors.faint)}>
                  {t(locale, "The app contacts Hugging Face only when you start a model download.")}
                </text>
              </div>
            </div>
          ) : null}

          <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}>
            {beat > 0 ? (
              <Button
                label={t(locale, "Back")}
                kind="secondary"
                onClick={() => setBeat(beat - 1)}
              />
            ) : null}
            {beat === 0 ? (
              <>
                <Button label={t(locale, "Continue")} onClick={() => setBeat(1)} />
                <Button
                  label={t(locale, "Continue in demo mode")}
                  kind="ghost"
                  onClick={() => onFinish()}
                />
              </>
            ) : null}
            {beat === 1 ? (
              <Button label={t(locale, "Continue")} onClick={() => setBeat(2)} />
            ) : null}
            {beat === 2 ? (
              <>
                {ready ? (
                  <Button label={t(locale, "Continue")} onClick={onFinish} />
                ) : models.downloading ? (
                  <Button
                    label={t(locale, "Cancel download")}
                    kind="danger"
                    onClick={onCancelDownload}
                  />
                ) : (
                  <Button
                    label={
                      models.error
                        ? t(locale, "Retry download")
                        : t(locale, "Download models (~%@)", "1.08 GB")
                    }
                    onClick={onDownload}
                  />
                )}
                <Button
                  label={t(locale, "Continue in demo mode")}
                  kind="ghost"
                  onClick={() => onFinish()}
                />
              </>
            ) : null}
            <div style={{ flexGrow: 1 }} />
            <Button label={t(locale, "Licenses")} kind="ghost" onClick={onOpenLicenses} />
          </div>

          <text style={{ ...textStyle(type.meta, colors.faint), lineHeight: 18 }}>
            {t(
              locale,
              "Audio is processed entirely on this device. Nothing is sent to any server. Transcripts are saved only when you export them.",
            )}
          </text>
        </Surface>
      </div>
    </div>
  );
}

function ModelProgress({ model, locale }: { model: ModelEntry; locale: string }) {
  const value = percent(model);
  const color =
    model.phase === "ready" ? colors.mint : model.phase === "error" ? colors.rose : colors.accent;
  const state =
    model.phase === "ready"
      ? t(locale, "Ready")
      : model.phase === "downloading"
        ? `${value}% · ${megabytes(model.doneBytes)} of ${megabytes(model.bytes)}`
        : model.phase === "error"
          ? t(locale, "stopped at %@%", value)
          : `${megabytes(model.bytes)}`;
  return (
    <div
      role="progressbar"
      aria-label={`${model.title}: ${state}`}
      aria-valuetext={`${value}%`}
      style={{
        padding: 14,
        borderRadius: 10,
        backgroundColor: "#FFFFFF07",
        borderWidth: 1,
        borderColor: colors.border,
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
    >
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 8 }}>
        <text style={{ ...textStyle(type.body, colors.text, 600), flexGrow: 1 }}>
          {model.title}
        </text>
        <text style={textStyle(type.meta, colors.muted, 600)}>{state}</text>
      </div>
      <div
        style={{
          height: 4,
          borderRadius: 2,
          backgroundColor: "#FFFFFF12",
          overflow: "hidden",
        }}
      >
        <div style={{ width: `${value}%`, height: 4, borderRadius: 2, backgroundColor: color }} />
      </div>
    </div>
  );
}
