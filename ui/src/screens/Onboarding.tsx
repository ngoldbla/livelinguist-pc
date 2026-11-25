import { useState } from "react";
import { BrandMark, Button, ProgressBar, StatusDot, Surface, Toast } from "../aurora/components";
import { LanguagePicker } from "../aurora/LanguagePicker";
import { colors, textStyle, type } from "../aurora/theme";
import { languageByCode, statusLabel } from "../i18n/languages";
import { t } from "../i18n/locales";
import { allReady, type ModelView, megabytes, percent } from "../store/modelStore";

/**
 * Mac first-run flow: welcome + privacy, language, download. The language pick
 * advances on its own; the download aggregates every model into one bar and
 * never names them.
 */
export function Onboarding({
  language,
  locale,
  onLanguage,
  onFinish,
  models,
  onDownload,
  onCancelDownload,
}: {
  language: string;
  locale: string;
  onLanguage: (code: string) => void;
  onFinish: () => void;
  models: ModelView;
  onDownload: () => void;
  onCancelDownload: () => void;
}) {
  const ready = allReady(models);
  const [beat, setBeat] = useState(Number(process.env.LL_ONBOARDING_BEAT ?? 0));
  const selected = languageByCode(language);
  const totalBytes = models.models.reduce((sum, model) => sum + model.bytes, 0);
  const doneBytes = models.models.reduce((sum, model) => sum + model.doneBytes, 0);
  const aggregate = totalBytes > 0 ? Math.min(100, Math.floor((doneBytes / totalBytes) * 100)) : 0;
  const firstPending = models.models.find((model) => model.phase !== "ready");
  const firstPercent = firstPending ? percent(firstPending) : aggregate;

  const back =
    beat > 0 ? (
      <Button label={t(locale, "Back")} kind="secondary" onClick={() => setBeat(beat - 1)} />
    ) : null;

  return (
    <div style={{ height: "100%", display: "flex", justifyContent: "center" }}>
      <div
        style={{
          width: "100%",
          maxWidth: 560,
          height: "100%",
          display: "flex",
          flexDirection: "column",
          gap: 16,
          padding: 28,
        }}
      >
        {beat === 0 ? (
          <>
            <div
              style={{
                flexGrow: 1,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: 22,
              }}
            >
              <BrandMark size={72} />
              <text
                style={{
                  ...textStyle(type.hero, colors.text, 700),
                  lineHeight: 40,
                  textAlign: "center",
                }}
              >
                {t(locale, "Welcome to Live Linguist")}
              </text>
              <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 8 }}>
                <text style={{ ...textStyle(type.meta, colors.mint, 700) }}>
                  {t(locale, "PRIVACY BY CONSTRUCTION")}
                </text>
                <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 21 }}>
                  {t(
                    locale,
                    "Audio is processed entirely on this device. Nothing is sent to any server. Transcripts are saved only when you export them.",
                  )}
                </text>
              </Surface>
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
              <Button
                label={t(locale, "Choose your language")}
                onClick={() => setBeat(1)}
                style={{ height: 48 }}
              />
              <Button label={t(locale, "Continue in demo mode")} kind="ghost" onClick={onFinish} />
            </div>
          </>
        ) : null}

        {beat === 1 ? (
          <>
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
              {back}
              <div style={{ flexGrow: 1 }} />
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <text style={{ ...textStyle(type.page, colors.text, 600), lineHeight: 32 }}>
                {t(locale, "Which language are you learning?")}
              </text>
              <text style={textStyle(type.body, colors.muted)}>
                {t(
                  locale,
                  "Live Linguist speaks %lld languages — captions, simplification, and the app itself.",
                  29,
                )}
              </text>
            </div>
            <LanguagePicker
              selected={language}
              onSelect={(code) => {
                onLanguage(code);
                setBeat(2);
              }}
              locale={locale}
              height="fill"
              showHeader={false}
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
          <>
            <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
              {back}
              <div style={{ flexGrow: 1 }} />
            </div>
            <Surface style={{ padding: 16, display: "flex", flexDirection: "column", gap: 4 }}>
              <text style={textStyle(type.body, colors.text, 600)}>
                {t(locale, "You're learning %@", selected.name)}
              </text>
              <text style={textStyle(type.meta, colors.faint)}>
                {statusLabel(selected.asrStatus)}
              </text>
            </Surface>

            <div style={{ flexGrow: 1, display: "flex", flexDirection: "column", gap: 14 }}>
              {ready ? null : (
                <text style={{ ...textStyle(type.body, colors.muted), lineHeight: 21 }}>
                  {t(
                    locale,
                    "Live captions need a one-time model download (about %@). Everything runs offline afterward.",
                    "1.08 GB",
                  )}
                </text>
              )}
              {models.downloading ? (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  <div style={{ display: "flex", flexDirection: "row" }}>
                    <text style={{ ...textStyle(type.body, colors.text, 600), flexGrow: 1 }}>
                      {t(locale, "Downloading models…")}
                    </text>
                    <text style={textStyle(type.meta, colors.muted, 600)}>{firstPercent}%</text>
                  </div>
                  <ProgressBar percent={aggregate > 0 ? aggregate : firstPercent} />
                  <text style={textStyle(type.meta, colors.faint)}>
                    {`${megabytes(doneBytes)} of ${megabytes(totalBytes)}`}
                  </text>
                </div>
              ) : null}
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
              {ready ? <StatusDot tone="mint" label={t(locale, "Models ready")} /> : null}
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
              {ready ? (
                <Button
                  label={t(locale, "Models installed — continue")}
                  onClick={onFinish}
                  style={{ height: 46 }}
                />
              ) : models.downloading ? (
                <Button
                  label={t(locale, "Cancel download")}
                  kind="danger"
                  onClick={onCancelDownload}
                  style={{ height: 46 }}
                />
              ) : (
                <Button
                  label={
                    models.error
                      ? t(locale, "Retry download")
                      : t(locale, "Download models (~%@)", "1.08 GB")
                  }
                  onClick={onDownload}
                  style={{ height: 46 }}
                />
              )}
              <Button label={t(locale, "Continue in demo mode")} kind="ghost" onClick={onFinish} />
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
}
