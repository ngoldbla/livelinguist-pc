import { Badge, Button, Divider, SectionHeader, Surface } from "../aurora/components";
import { colors, textStyle } from "../aurora/theme";
import { t } from "../i18n/locales";

export function LicensesScreen({ locale, onBack }: { locale: string; onBack: () => void }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, padding: 28, height: "100%" }}>
      <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 12 }}>
        <Button label={t(locale, "Back")} kind="secondary" onClick={onBack} />
        <div style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1 }}>
          <text style={{ ...textStyle(10, colors.accent, 700) }}>
            {t(locale, "LICENSES").toUpperCase()}
          </text>
          <text style={textStyle(24, colors.text, 700)}>{t(locale, "Licenses")}</text>
        </div>
        <Badge label="Apache-2.0 · MIT" tone="mint" />
      </div>

      <div
        style={{
          flexGrow: 1,
          minHeight: 0,
          overflow: "scroll",
          scrollbarColor: "#FFFFFF22",
          scrollbarWidth: 6,
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <LicenseBlock
          title={t(locale, "MODELS")}
          rows={[
            {
              name: "Qwen3 0.6B / 1.7B EasyLanguage",
              detail: "Qwen3, Apache-2.0; fine-tuned EasyLanguage checkpoints",
              tag: "Apache-2.0",
            },
            {
              name: "Nemotron 3.5 ASR 0.6B",
              detail: "NVIDIA Nemotron; license must be resolved before publishing the ONNX export",
              tag: "resolve upstream",
            },
            { name: "Silero VAD", detail: "sherpa-onnx bundle", tag: "MIT (verify)" },
          ]}
        />
        <LicenseBlock
          title={t(locale, "FRAMEWORKS")}
          rows={[
            { name: "GPUIX 0.11", detail: "React bindings for GPUI", tag: "Apache-2.0" },
            { name: "GPUI", detail: "Zed's GPU UI framework", tag: "Apache-2.0" },
            { name: "llama.cpp", detail: "GGUF inference", tag: "MIT" },
            { name: "sherpa-onnx", detail: "streaming ASR", tag: "Apache-2.0" },
            { name: "React 19.2", detail: "UI runtime", tag: "MIT" },
            {
              name: "Rust crates",
              detail: "serde, tokio, reqwest, ringbuf, rubato",
              tag: "per crate",
            },
          ]}
        />
        <LicenseBlock
          title={t(locale, "FRAMEWORK PACKS")}
          rows={[
            {
              name: "FALC (fr)",
              detail: "UNAPEI easy-to-read rules; ported verbatim from the Mac app",
              tag: "product data",
            },
            {
              name: "Leichte Sprache (de)",
              detail: "Netzwerk Leichte Sprache / DIN SPEC 33429 orientation",
              tag: "product data",
            },
            { name: "Easy English (en)", detail: "Plain English guidance", tag: "product data" },
            { name: "Lectura Fácil (es)", detail: "Lectura Fácil guidance", tag: "product data" },
            {
              name: "25 further locales",
              detail: "National easy-to-read standards in each pack",
              tag: "product data",
            },
          ]}
        />
        <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 8 }}>
          <SectionHeader
            title={t(locale, "About")}
            subtitle={t(locale, "Built for Windows 11 x64")}
          />
          <text style={textStyle(12, colors.muted)}>
            {t(locale, "Linux dev preview")} · Live Linguist PC 0.1.0 · GPUIX 0.11.0 · React 19.2.3.
          </text>
          <Divider />
          <text style={textStyle(11, colors.faint)}>
            {t(
              locale,
              "No telemetry. No cloud inference. Model downloads contact Hugging Face only when you start one.",
            )}
          </text>
        </Surface>
      </div>
    </div>
  );
}

function LicenseBlock({
  title,
  rows,
}: {
  title: string;
  rows: Array<{ name: string; detail: string; tag: string }>;
}) {
  return (
    <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 10 }}>
      <text style={{ ...textStyle(10, colors.faint, 700) }}>{title}</text>
      {rows.map((row) => (
        <div
          key={row.name}
          style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}
        >
          <div
            style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1, minWidth: 0 }}
          >
            <text style={textStyle(12, colors.text, 600)}>{row.name}</text>
            <text style={textStyle(10, colors.faint)}>{row.detail}</text>
          </div>
          <Badge label={row.tag} tone="neutral" />
        </div>
      ))}
    </Surface>
  );
}
