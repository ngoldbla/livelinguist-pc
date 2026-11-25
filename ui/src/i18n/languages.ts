export type AsrStatus = "ok" | "degraded" | "simplify_only" | "tbd";

export interface LanguageOption {
  code: string;
  uiCode: string;
  name: string;
  shortName: string;
  rtl?: boolean;
  /// Mirrors `CoverageMatrix::v1()` in `core/crates/ll-core/src/coverage.rs`
  /// (measured in docs/qa/2026-11-23-asr-coverage-v1.md); a test keeps them equal.
  asrStatus: AsrStatus;
  pack: string;
}

export const LANGUAGES: LanguageOption[] = [
  {
    code: "de-DE",
    uiCode: "de",
    name: "Deutsch (Deutschland)",
    shortName: "Deutsch",
    asrStatus: "ok",
    pack: "leichte_sprache",
  },
  {
    code: "fr-FR",
    uiCode: "fr",
    name: "Français (France)",
    shortName: "Français",
    asrStatus: "ok",
    pack: "falc",
  },
  {
    code: "es-ES",
    uiCode: "es",
    name: "Español (España)",
    shortName: "Español",
    asrStatus: "ok",
    pack: "lectura_facil",
  },
  {
    code: "en-US",
    uiCode: "en",
    name: "English (US)",
    shortName: "English",
    asrStatus: "ok",
    pack: "easy_english",
  },
  {
    code: "ar-SA",
    uiCode: "ar",
    name: "العربية (Arabic)",
    shortName: "العربية",
    rtl: true,
    asrStatus: "ok",
    pack: "ar",
  },
  {
    code: "da-DK",
    uiCode: "da",
    name: "Dansk (Danish)",
    shortName: "Dansk",
    asrStatus: "simplify_only",
    pack: "da",
  },
  {
    code: "et-EE",
    uiCode: "et",
    name: "Eesti (Estonian)",
    shortName: "Eesti",
    asrStatus: "simplify_only",
    pack: "et",
  },
  {
    code: "fi-FI",
    uiCode: "fi",
    name: "Suomi (Finnish)",
    shortName: "Suomi",
    asrStatus: "simplify_only",
    pack: "fi",
  },
  {
    code: "hi-IN",
    uiCode: "hi",
    name: "हिन्दी (Hindi)",
    shortName: "हिन्दी",
    asrStatus: "ok",
    pack: "hi",
  },
  {
    code: "it-IT",
    uiCode: "it",
    name: "Italiano (Italian)",
    shortName: "Italiano",
    asrStatus: "ok",
    pack: "it",
  },
  {
    code: "ja-JP",
    uiCode: "ja",
    name: "日本語 (Japanese)",
    shortName: "日本語",
    asrStatus: "simplify_only",
    pack: "ja",
  },
  {
    code: "ko-KR",
    uiCode: "ko",
    name: "한국어 (Korean)",
    shortName: "한국어",
    asrStatus: "ok",
    pack: "ko",
  },
  {
    code: "nl-NL",
    uiCode: "nl",
    name: "Nederlands (Dutch)",
    shortName: "Nederlands",
    asrStatus: "simplify_only",
    pack: "nl",
  },
  {
    code: "pt-BR",
    uiCode: "pt-BR",
    name: "Português (Brasil)",
    shortName: "Português (BR)",
    asrStatus: "ok",
    pack: "pt-BR",
  },
  {
    code: "pt-PT",
    uiCode: "pt-PT",
    name: "Português (Portugal)",
    shortName: "Português (PT)",
    asrStatus: "ok",
    pack: "pt-PT",
  },
  {
    code: "ru-RU",
    uiCode: "ru",
    name: "Русский (Russian)",
    shortName: "Русский",
    asrStatus: "ok",
    pack: "ru",
  },
  {
    code: "sk-SK",
    uiCode: "sk",
    name: "Slovenčina (Slovak)",
    shortName: "Slovenčina",
    asrStatus: "simplify_only",
    pack: "sk",
  },
  {
    code: "sv-SE",
    uiCode: "sv",
    name: "Svenska (Swedish)",
    shortName: "Svenska",
    asrStatus: "simplify_only",
    pack: "sv",
  },
  {
    code: "tr-TR",
    uiCode: "tr",
    name: "Türkçe (Turkish)",
    shortName: "Türkçe",
    asrStatus: "simplify_only",
    pack: "tr",
  },
  {
    code: "vi-VN",
    uiCode: "vi",
    name: "Tiếng Việt (Vietnamese)",
    shortName: "Tiếng Việt",
    asrStatus: "ok",
    pack: "vi",
  },
  {
    code: "zh-CN",
    uiCode: "zh-Hans",
    name: "中文 (Mandarin, Simplified)",
    shortName: "中文",
    asrStatus: "simplify_only",
    pack: "zh-CN",
  },
  {
    code: "uk-UA",
    uiCode: "uk",
    name: "Українська (Ukrainian)",
    shortName: "Українська",
    asrStatus: "simplify_only",
    pack: "uk",
  },
  {
    code: "ro-RO",
    uiCode: "ro",
    name: "Română (Romanian)",
    shortName: "Română",
    asrStatus: "simplify_only",
    pack: "ro",
  },
  {
    code: "bg-BG",
    uiCode: "bg",
    name: "Български (Bulgarian)",
    shortName: "Български",
    asrStatus: "simplify_only",
    pack: "bg",
  },
  {
    code: "pl-PL",
    uiCode: "pl",
    name: "Polski (Polish)",
    shortName: "Polski",
    asrStatus: "simplify_only",
    pack: "pl",
  },
  {
    code: "hr-HR",
    uiCode: "hr",
    name: "Hrvatski (Croatian)",
    shortName: "Hrvatski",
    asrStatus: "simplify_only",
    pack: "hr",
  },
  {
    code: "nb-NO",
    uiCode: "nb",
    name: "Norsk bokmål (Norwegian)",
    shortName: "Norsk",
    asrStatus: "simplify_only",
    pack: "nb",
  },
  {
    code: "hu-HU",
    uiCode: "hu",
    name: "Magyar (Hungarian)",
    shortName: "Magyar",
    asrStatus: "simplify_only",
    pack: "hu",
  },
  {
    code: "cs-CZ",
    uiCode: "cs",
    name: "Čeština (Czech)",
    shortName: "Čeština",
    asrStatus: "simplify_only",
    pack: "cs",
  },
];

export function languageByCode(code: string): LanguageOption {
  return LANGUAGES.find((language) => language.code === code) ?? LANGUAGES[1]!;
}

export function languageByUiCode(code: string): LanguageOption {
  return LANGUAGES.find((language) => language.uiCode === code) ?? LANGUAGES[1]!;
}

export function statusLabel(status: AsrStatus): string {
  switch (status) {
    case "ok":
      return "Live captions";
    case "degraded":
      return "Live captions · reduced";
    case "tbd":
      return "Not measured";
    default:
      return "Simplified text only";
  }
}

/** One plain sentence explaining what the status means for the user. */
export function statusDetail(status: AsrStatus): string {
  switch (status) {
    case "ok":
      return "Captions come straight from what is said.";
    case "degraded":
      return "Captions work, but recognition is less accurate for this language.";
    case "tbd":
      return "Live captions for this language have not been measured yet.";
    default:
      return "No live captions yet. Paste or type text to simplify it.";
  }
}
