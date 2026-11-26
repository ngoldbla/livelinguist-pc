import { Badge, Divider, Surface } from "../aurora/components";
import { colors, textStyle, type } from "../aurora/theme";
import { t } from "../i18n/locales";
import { APP_VERSION } from "../version";

/**
 * Attribution content, shared by the Settings Licenses tab. No page chrome:
 * the tab provides the title and the scroll surface.
 */
export function LicensesContent({ locale }: { locale: string }) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 12,
        alignItems: "flex-start",
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: 640,
          display: "flex",
          flexDirection: "column",
          gap: 12,
        }}
      >
        <LicenseBlock
          title={t(locale, "MODELS")}
          rows={[
            {
              name: "Qwen3 0.6B EasyLanguage",
              detail: "LoRA fine-tune of Qwen3 0.6B for same-language simplification; GGUF Q4_K_M",
              tag: "Apache-2.0",
            },
            {
              name: "Nemotron 3.5 ASR 0.6B",
              detail: "NVIDIA multilingual streaming ASR; int8 conversion by sherpa-onnx",
              tag: "NVIDIA Open Model",
            },
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
            { name: "Lucide", detail: "interface icon set (SVG)", tag: "ISC" },
            { name: "M PLUS Rounded 1c", detail: "rounded caption face", tag: "OFL-1.1" },
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
        <Surface style={{ padding: 18, display: "flex", flexDirection: "column", gap: 10 }}>
          <text style={{ ...textStyle(type.title, colors.text, 600) }}>{t(locale, "About")}</text>
          <InfoRow label={t(locale, "Version")} value={APP_VERSION} />
          <InfoRow
            label={t(locale, "Built for Windows 11 x64")}
            value="GPUIX 0.11.0 · React 19.2.3"
          />
          <Divider />
          <text style={{ ...textStyle(type.meta, colors.muted), lineHeight: 18 }}>
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

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 16 }}>
      <text style={{ ...textStyle(type.meta, colors.faint), width: 200 }}>{label}</text>
      <text style={{ ...textStyle(type.body, colors.text), flexGrow: 1, minWidth: 0 }}>
        {value}
      </text>
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
      <text style={{ ...textStyle(type.meta, colors.faint, 600) }}>{title}</text>
      {rows.map((row) => (
        <div
          key={row.name}
          style={{ display: "flex", flexDirection: "row", alignItems: "center", gap: 10 }}
        >
          <div
            style={{ display: "flex", flexDirection: "column", gap: 2, flexGrow: 1, minWidth: 0 }}
          >
            <text style={textStyle(type.body, colors.text, 600)}>{row.name}</text>
            <text style={textStyle(type.meta, colors.faint)}>{row.detail}</text>
          </div>
          <Badge label={row.tag} tone="neutral" />
        </div>
      ))}
    </Surface>
  );
}
