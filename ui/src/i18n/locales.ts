// Generated locale catalogs (English keys + 28 translations from the Mac
// Localizable.xcstrings, plus desktop-only keys). Do not hand-edit a locale
// file for behavior; update the key in every catalog intentionally.

import l_ar from "../../../assets/locales/ar.json";
import l_bg from "../../../assets/locales/bg.json";
import l_cs from "../../../assets/locales/cs.json";
import l_da from "../../../assets/locales/da.json";
import l_de from "../../../assets/locales/de.json";
import en from "../../../assets/locales/en.json";
import l_es from "../../../assets/locales/es.json";
import l_et from "../../../assets/locales/et.json";
import l_fi from "../../../assets/locales/fi.json";
import l_fr from "../../../assets/locales/fr.json";
import l_hi from "../../../assets/locales/hi.json";
import l_hr from "../../../assets/locales/hr.json";
import l_hu from "../../../assets/locales/hu.json";
import l_it from "../../../assets/locales/it.json";
import l_ja from "../../../assets/locales/ja.json";
import l_ko from "../../../assets/locales/ko.json";
import l_nb from "../../../assets/locales/nb.json";
import l_nl from "../../../assets/locales/nl.json";
import l_pl from "../../../assets/locales/pl.json";
import l_pt_BR from "../../../assets/locales/pt-BR.json";
import l_pt_PT from "../../../assets/locales/pt-PT.json";
import l_ro from "../../../assets/locales/ro.json";
import l_ru from "../../../assets/locales/ru.json";
import l_sk from "../../../assets/locales/sk.json";
import l_sv from "../../../assets/locales/sv.json";
import l_tr from "../../../assets/locales/tr.json";
import l_uk from "../../../assets/locales/uk.json";
import l_vi from "../../../assets/locales/vi.json";
import l_zh_Hans from "../../../assets/locales/zh-Hans.json";

export const catalogs: Record<string, Record<string, string>> = {
  ar: l_ar as Record<string, string>,
  bg: l_bg as Record<string, string>,
  cs: l_cs as Record<string, string>,
  da: l_da as Record<string, string>,
  de: l_de as Record<string, string>,
  en: en as Record<string, string>,
  es: l_es as Record<string, string>,
  et: l_et as Record<string, string>,
  fi: l_fi as Record<string, string>,
  fr: l_fr as Record<string, string>,
  hi: l_hi as Record<string, string>,
  hr: l_hr as Record<string, string>,
  hu: l_hu as Record<string, string>,
  it: l_it as Record<string, string>,
  ja: l_ja as Record<string, string>,
  ko: l_ko as Record<string, string>,
  nb: l_nb as Record<string, string>,
  nl: l_nl as Record<string, string>,
  pl: l_pl as Record<string, string>,
  "pt-BR": l_pt_BR as Record<string, string>,
  "pt-PT": l_pt_PT as Record<string, string>,
  ro: l_ro as Record<string, string>,
  ru: l_ru as Record<string, string>,
  sk: l_sk as Record<string, string>,
  sv: l_sv as Record<string, string>,
  tr: l_tr as Record<string, string>,
  uk: l_uk as Record<string, string>,
  vi: l_vi as Record<string, string>,
  "zh-Hans": l_zh_Hans as Record<string, string>,
};

export const AVAILABLE_LOCALES = Object.keys(catalogs);

export type MessageKey = keyof typeof en;

export function t(
  locale: string,
  key: MessageKey | string,
  ...args: Array<string | number>
): string {
  const english = catalogs["en"]!;
  const catalog = catalogs[locale] ?? english;
  const value = catalog[key] ?? english[key] ?? String(key);
  let index = 0;
  return value.replace(/%%|%lld|%@/g, (match) => {
    if (match === "%%") return "%";
    const next = args[index];
    index += 1;
    return next === undefined ? "" : String(next);
  });
}

/// Resolve the UI locale: explicit learning language -> device -> English.
export function resolveLocale(
  followLearningLanguage: boolean,
  chosenCode: string | null,
  devicePreferred: string[],
  available: string[] = AVAILABLE_LOCALES,
): string {
  const candidates: string[] = [];
  if (followLearningLanguage && chosenCode) candidates.push(chosenCode);
  candidates.push(...devicePreferred, "en");
  for (const candidate of candidates) {
    if (available.includes(candidate)) return candidate;
    const base = candidate.split("-")[0];
    if (base && available.includes(base)) return base;
  }
  return "en";
}

export function isRtl(code: string): boolean {
  return /^(ar|he|fa|ur)(-|$)/.test(code);
}

export function deviceLocales(): string[] {
  const env = Intl.DateTimeFormat().resolvedOptions().locale;
  return env ? [env] : ["en"];
}
